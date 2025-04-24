use std::{collections::HashMap, sync::Arc, time::Duration};

use eyre::Result;
use rand::{Rng, distr::Alphanumeric};
use rumqttc::{AsyncClient, Event, MqttOptions, QoS};
use tokio::sync::RwLock;

use crate::settings::Settings;

#[derive(Debug, Default)]
struct CommandState {
    last_value: Option<serde_json::Value>,
}

#[derive(Default)]
struct TopicState {
    commands: HashMap<String, CommandState>,
}

struct State {
    topics: HashMap<String, TopicState>,
}

async fn handle_mqtt_event(
    event: &Event,
    client: &AsyncClient,
    state: &Arc<RwLock<State>>,
    settings: &Settings,
) -> Result<()> {
    let settings = settings.clone();
    let client = client.clone();

    match event {
        // Subscribe to all configured topics when connected
        rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_)) => {
            for topic_settings in settings.topics.values() {
                client
                    .subscribe(&topic_settings.topic, QoS::AtMostOnce)
                    .await?;
            }
        }

        rumqttc::Event::Incoming(rumqttc::Packet::Publish(msg)) => {
            let topic = msg.topic.clone();
            let payload = msg.payload.clone();

            if let Some(topic_settings) = settings.topics.values().find(|s| s.topic == topic) {
                for (cmd_name, cmd_settings) in &topic_settings.commands {
                    let rule = &cmd_settings.rule;
                    let ignore_unchanged = rule.ignore_unchanged.unwrap_or(true);

                    let value = match &rule.ptr {
                        Some(ptr) => {
                            let payload: serde_json::Value = serde_json::from_slice(&payload)?;
                            ptr.resolve(&payload)?.clone()
                        }
                        None => serde_json::from_slice(&payload)?,
                    };

                    let last_value = {
                        let mut state = state.write().await;
                        let command_state = state
                            .topics
                            .entry(topic.clone())
                            .or_default()
                            .commands
                            .entry(cmd_name.clone())
                            .or_default();

                        let last_value = command_state.last_value.clone();
                        command_state.last_value = Some(value.clone());
                        last_value
                    };

                    if (!ignore_unchanged || (last_value.as_ref() != Some(&value)))
                        && (rule.matching_value.is_none()
                            || rule.matching_value.as_ref() == Some(&value))
                    {
                        // Execute command
                        tokio::process::Command::new(&cmd_settings.cmd)
                            .args(&cmd_settings.args)
                            .spawn()?;
                    }
                }
            }
        }
        _ => {}
    }

    Ok(())
}

pub async fn start_mqtt_loop(settings: &Settings) -> Result<()> {
    let random_string: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();

    let mut options = MqttOptions::new(
        format!("{}-{}", settings.mqtt.id.clone(), random_string),
        settings.mqtt.host.clone(),
        settings.mqtt.port,
    );
    options.set_keep_alive(Duration::from_secs(5));

    // Set credentials if provided
    if let Some(username) = &settings.mqtt.username {
        options.set_credentials(
            username,
            settings.mqtt.password.as_deref().unwrap_or_default(),
        );
    }
    let (client, mut eventloop) = AsyncClient::new(options, 10);

    {
        let client = client.clone();
        let settings = settings.clone();
        tokio::task::spawn(async move {
            let state = Arc::new(RwLock::new(State {
                topics: HashMap::new(),
            }));

            loop {
                {
                    let event = match eventloop.poll().await {
                        Ok(event) => event,
                        Err(e) => {
                            eprintln!("MQTT connection error: {:?}", e);
                            tokio::time::sleep(Duration::from_millis(1000)).await;
                            continue;
                        }
                    };

                    let res = handle_mqtt_event(&event, &client, &state, &settings).await;

                    if let Err(e) = res {
                        eprintln!("MQTT event handler error: {:?}", e);
                        tokio::time::sleep(Duration::from_millis(1000)).await;
                    }
                }

                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
    }

    Ok(())
}
