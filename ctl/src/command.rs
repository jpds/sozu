use sozu::channel::Channel;
use sozu::messages::{Order, Instance};
use sozu_command::data::{AnswerData,ConfigCommand,ConfigMessage,ConfigMessageAnswer,ConfigMessageStatus,RunState};

use std::collections::HashSet;
use rand::{thread_rng, Rng};

fn generate_id() -> String {
  let s: String = thread_rng().gen_ascii_chars().take(6).collect();
  format!("ID-{}", s)
}

pub fn save_state(channel: &mut Channel<ConfigMessage,ConfigMessageAnswer>, path: &str) {
  let id = generate_id();
  channel.write_message(&ConfigMessage::new(
    id.clone(),
    ConfigCommand::SaveState(path.to_string()),
    None,
  ));

  match channel.read_message() {
    None          => println!("the proxy didn't answer"),
    Some(message) => {
      if id != message.id {
        println!("received message with invalid id: {:?}", message);
        return;
      }
      match message.status {
        ConfigMessageStatus::Processing => {
          // do nothing here
          // for other messages, we would loop over read_message
          // until an error or ok message was sent
        },
        ConfigMessageStatus::Error => {
          println!("could not save proxy state: {}", message.message);
        },
        ConfigMessageStatus::Ok => {
          println!("Proxy state saved to {}", path);
        }
      }
    }
  }
}

pub fn load_state(channel: &mut Channel<ConfigMessage,ConfigMessageAnswer>, path: &str) {
  let id = generate_id();
  channel.write_message(&ConfigMessage::new(
    id.clone(),
    ConfigCommand::LoadState(path.to_string()),
    None,
  ));

  match channel.read_message() {
    None          => println!("the proxy didn't answer"),
    Some(message) => {
      if id != message.id {
        println!("received message with invalid id: {:?}", message);
        return;
      }
      match message.status {
        ConfigMessageStatus::Processing => {
          // do nothing here
          // for other messages, we would loop over read_message
          // until an error or ok message was sent
        },
        ConfigMessageStatus::Error => {
          println!("could not save proxy state: {}", message.message);
        },
        ConfigMessageStatus::Ok => {
          println!("Proxy state saved to {}", path);
        }
      }
    }
  }
}

pub fn dump_state(channel: &mut Channel<ConfigMessage,ConfigMessageAnswer>) {
  let id = generate_id();
  channel.write_message(&ConfigMessage::new(
    id.clone(),
    ConfigCommand::DumpState,
    None,
  ));

  match channel.read_message() {
    None          => println!("the proxy didn't answer"),
    Some(message) => {
      if id != message.id {
        println!("received message with invalid id: {:?}", message);
        return;
      }
      match message.status {
        ConfigMessageStatus::Processing => {
          // do nothing here
          // for other messages, we would loop over read_message
          // until an error or ok message was sent
        },
        ConfigMessageStatus::Error => {
          println!("could not dump proxy state: {}", message.message);
        },
        ConfigMessageStatus::Ok => {
          println!("Proxy state:\n{}", message.message);
        }
      }
    }
  }
}

pub fn soft_stop(channel: &mut Channel<ConfigMessage,ConfigMessageAnswer>) {
  println!("shutting down proxy");
  let id = generate_id();
  channel.write_message(&ConfigMessage::new(
    id.clone(),
    ConfigCommand::ProxyConfiguration(Order::SoftStop),
    //FIXME: we should be able to soft stop one specific worker
    None,
  ));

  loop {
    match channel.read_message() {
      None          => println!("the proxy didn't answer"),
      Some(message) => {
        if &id != &message.id {
          println!("received message with invalid id: {:?}", message);
          return;
        }
        match message.status {
          ConfigMessageStatus::Processing => {
            println!("Proxy is processing: {}", message.message);
          },
          ConfigMessageStatus::Error => {
            println!("could not stop the proxy: {}", message.message);
          },
          ConfigMessageStatus::Ok => {
            println!("Proxy shut down: {}", message.message);
            break;
          }
        }
      }
    }
  }
}

pub fn hard_stop(channel: &mut Channel<ConfigMessage,ConfigMessageAnswer>) {
  println!("shutting down proxy");
  let id = generate_id();
  channel.write_message(&ConfigMessage::new(
    id.clone(),
    ConfigCommand::ProxyConfiguration(Order::HardStop),
    //FIXME: we should be able to soft stop one specific worker
    None,
  ));

  loop {
    match channel.read_message() {
      None          => println!("the proxy didn't answer"),
      Some(message) => {
        match message.status {
          ConfigMessageStatus::Processing => {
            println!("Proxy is processing: {}", message.message);
          },
          ConfigMessageStatus::Error => {
            println!("could not stop the proxy: {}", message.message);
          },
          ConfigMessageStatus::Ok => {
            if &id == &message.id {
              println!("Proxy shut down: {}", message.message);
              break;
            }
          }
        }
      }
    }
  }
}

pub fn upgrade(channel: &mut Channel<ConfigMessage,ConfigMessageAnswer>) {
  let id = generate_id();
  channel.write_message(&ConfigMessage::new(
    id.clone(),
    ConfigCommand::ListWorkers,
    None,
  ));

  match channel.read_message() {
    None          => println!("the proxy didn't answer"),
    Some(message) => {
      if id != message.id {
        println!("received message with invalid id: {:?}", message);
        return;
      }
      match message.status {
        ConfigMessageStatus::Processing => {
          println!("should have obtained an answer immediately");
          return;
        },
        ConfigMessageStatus::Error => {
          println!("could not get the worker list: {}", message.message);
          return
        },
        ConfigMessageStatus::Ok => {
          println!("Worker list:\n{:?}", message.data);
          if let Some(AnswerData::Workers(ref workers)) = message.data {
            let mut launching: HashSet<String> = HashSet::new();
            let mut stopping:  HashSet<String> = HashSet::new();

            for ref worker in workers.iter().filter(|worker| worker.run_state == RunState::Running) {
              let id = generate_id();
              let msg = ConfigMessage::new(
                id.clone(),
                ConfigCommand::LaunchWorker("BLAH".to_string()),
                None,
              );
              println!("sending message: {:?}", msg);
              channel.write_message(&msg);
              launching.insert(id);
            }

            for ref worker in workers.iter().filter(|worker| worker.run_state == RunState::Running) {
              let id = generate_id();
              let msg = ConfigMessage::new(
                id.clone(),
                ConfigCommand::ProxyConfiguration(Order::SoftStop),
                Some(worker.id),
              );
              println!("sending message: {:?}", msg);
              channel.write_message(&msg);
              stopping.insert(id);
            }


            loop {
              println!("launching: {:?}\nstopping: {:?}", launching, stopping);
              if launching.is_empty() && stopping.is_empty() {
                break;
              }
              match channel.read_message() {
                None          => println!("the proxy didn't answer"),
                Some(message) => {
                  println!("received message: {:?}", message);
                  match message.status {
                    ConfigMessageStatus::Processing => {
                    },
                    ConfigMessageStatus::Error => {
                      println!("error for message[{}]: {}", message.id, message.message);
                      if launching.contains(&message.id) {
                        launching.remove(&message.id);
                        println!("launch message with ID {} done", message.id);
                      }
                      if stopping.contains(&message.id) {
                        stopping.remove(&message.id);
                        println!("stop message with ID {} done", message.id);
                      }
                    },
                    ConfigMessageStatus::Ok => {
                      if launching.contains(&message.id) {
                        launching.remove(&message.id);
                        println!("launch message with ID {} done", message.id);
                      }
                      if stopping.contains(&message.id) {
                        stopping.remove(&message.id);
                        println!("stop message with ID {} done", message.id);
                      }
                    }
                  }
                }
              }
            }

            println!("worker upgrade done");
            let id = generate_id();
            channel.write_message(&ConfigMessage::new(
              id.clone(),
              ConfigCommand::UpgradeMaster,
              None,
            ));

            loop {
              match channel.read_message() {
                None          => println!("the proxy didn't answer"),
                Some(message) => {
                  if &id != &message.id {
                    println!("received message with invalid id: {:?}", message);
                    return;
                  }
                  match message.status {
                    ConfigMessageStatus::Processing => {
                      println!("master is processing: {}", message.message);
                    },
                    ConfigMessageStatus::Error => {
                      println!("could not upgrade the master: {}", message.message);
                      return;
                    },
                    ConfigMessageStatus::Ok => {
                      println!("successfully upgraded the master: {}", message.message);
                      return;
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}

pub fn status(channel: &mut Channel<ConfigMessage,ConfigMessageAnswer>) {
  let id = generate_id();
  channel.write_message(&ConfigMessage::new(
    id.clone(),
    ConfigCommand::ListWorkers,
    None,
  ));

  match channel.read_message() {
    None          => println!("the proxy didn't answer"),
    Some(message) => {
      if id != message.id {
        println!("received message with invalid id: {:?}", message);
        return;
      }
      match message.status {
        ConfigMessageStatus::Processing => {
          println!("should have obtained an answer immediately");
          return;
        },
        ConfigMessageStatus::Error => {
          println!("could not get the worker list: {}", message.message);
          return
        },
        ConfigMessageStatus::Ok => {
          println!("Worker list:\n{:?}", message.data);
          if let Some(AnswerData::Workers(ref workers)) = message.data {
            let mut expecting: HashSet<String> = HashSet::new();

            for ref worker in workers.iter().filter(|worker| worker.run_state == RunState::Running) {
              let id = generate_id();
              let msg = ConfigMessage::new(
                id.clone(),
                ConfigCommand::ProxyConfiguration(Order::Status),
                Some(worker.id),
              );
              println!("sending message: {:?}", msg);
              channel.write_message(&msg);
              expecting.insert(id);
            }


            loop {
              println!("expecting: {:?}", expecting);
              if expecting.is_empty() {
                break;
              }
              match channel.read_message() {
                None          => println!("the proxy didn't answer"),
                Some(message) => {
                  println!("received message: {:?}", message);
                  match message.status {
                    ConfigMessageStatus::Processing => {
                    },
                    ConfigMessageStatus::Error => {
                      println!("error for message[{}]: {}", message.id, message.message);
                      if expecting.contains(&message.id) {
                        expecting.remove(&message.id);
                        println!("status message with ID {} done", message.id);
                      }
                    },
                    ConfigMessageStatus::Ok => {
                      if expecting.contains(&message.id) {
                        expecting.remove(&message.id);
                        println!("status message with ID {} done", message.id);
                      }
                    }
                  }
                }
              }
            }

            println!("worker upgrade done");
          }
        }
      }
    }
  }
}

pub fn remove_backend(channel: &mut Channel<ConfigMessage,ConfigMessageAnswer>, app_id: &str, ip: &str, port: u16) {
  let id = generate_id();
  channel.write_message(&ConfigMessage::new(
    id.clone(),
    ConfigCommand::ProxyConfiguration(Order::RemoveInstance(Instance {
      app_id: String::from(app_id),
      ip_address: String::from(ip),
      port: port
    })),
    None,
  ));

  match channel.read_message() {
    None          => println!("the proxy didn't answer"),
    Some(message) => {
      if id != message.id {
        println!("received message with invalid id: {:?}", message);
        return;
      }
      match message.status {
        ConfigMessageStatus::Processing => {
          // do nothing here
          // for other messages, we would loop over read_message
          // until an error or ok message was sent
        },
        ConfigMessageStatus::Error => {
          println!("could not remove backend : {}", message.message);
        },
        ConfigMessageStatus::Ok => {
          println!("backend {}:{} removed for app : {} ", ip, port, app_id);
        }
      }
    }
  }
}


pub fn add_backend(channel: &mut Channel<ConfigMessage,ConfigMessageAnswer>, app_id: &str, ip: &str, port: u16) {
  let id = generate_id();
  channel.write_message(&ConfigMessage::new(
    id.clone(),
    ConfigCommand::ProxyConfiguration(Order::AddInstance(Instance {
      app_id: String::from(app_id),
      ip_address: String::from(ip),
      port: port
    })),
    None,
  ));

  match channel.read_message() {
    None          => println!("the proxy didn't answer"),
    Some(message) => {
      if id != message.id {
        println!("received message with invalid id: {:?}", message);
        return;
      }
      match message.status {
        ConfigMessageStatus::Processing => {
          // do nothing here
          // for other messages, we would loop over read_message
          // until an error or ok message was sent
        },
        ConfigMessageStatus::Error => {
          println!("could not add backend : {}", message.message);
        },
        ConfigMessageStatus::Ok => {
          println!("backend {}:{} added for app : {} ", ip, port, app_id);
        }
      }
    }
  }
}
