#![allow(unused)]

use bevy::prelude::*;
use outcome_core::{arraystring, Address, Sim};
use outcome_net::{Client, ClientConfig, Worker};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Enables communication with a simulation server from the level of a Bevy
/// application.
pub struct OutcomeClientPlugin {
    server: String,
}

impl OutcomeClientPlugin {
    pub fn with_server_addr(addr: &str) -> Self {
        Self {
            server: addr.to_string(),
        }
    }
}

pub struct OutcomeClientResource {
    pub client: Arc<Mutex<outcome_net::Client>>,
}

impl Plugin for OutcomeClientPlugin {
    fn build(&self, app: &mut AppBuilder) {
        // create a new blocking client
        //let mut client = Client::new("name", true, false, None, Some(1000)).unwrap();
        let config = ClientConfig {
            heartbeat: None,
            //encodings: vec![Encoding::Bincode],
            //transports: vec![Transport::Tcp],
            ..Default::default()
        };
        let mut client = Client::new_with_config(None, config).unwrap();
        client.connect(self.server.clone(), None).unwrap();

        //app.add_resource(OutcomeClientResource { client });
        //app.add_thread_local_resource(OutcomeClientResource {
        //client: Arc::new(Mutex::new(client)),
        //});
        app.add_resource(OutcomeClientResource {
            client: Arc::new(Mutex::new(client)),
        });
        // .add_system_to_stage(stage::PRE_UPDATE, frame_update.system());
    }
}

pub struct SyncMarker(pub Option<u32>);

/// Worker plugin.
pub struct OutcomeWorkerPlugin;

pub struct OutcomeWorkerResource {
    worker: Worker,
}

impl Plugin for OutcomeWorkerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let res = OutcomeWorkerResource {
            worker: Worker::new("127.0.0.1:3131").unwrap(),
        };

        app.add_thread_local_resource(res);

        // app.add_event::<NetworkEvent>()
        //     .add_system(process_network_events.system());
        // unimplemented!()
    }
}

/// Local sim plugin.
pub struct OutcomeSimPlugin {
    source: OutcomeSimSource,
}

impl OutcomeSimPlugin {
    pub fn with_scenario(path: &str) -> Self {
        Self {
            source: OutcomeSimSource::Scenario(path.to_string()),
        }
    }

    pub fn with_snapshot(path: &str) -> Self {
        Self {
            source: OutcomeSimSource::Snapshot(path.to_string()),
        }
    }
}

enum OutcomeSimSource {
    Scenario(String),
    Snapshot(String),
}

pub struct OutcomeSimResource {
    sim: Sim,
}

impl Plugin for OutcomeSimPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let sim = match &self.source {
            OutcomeSimSource::Scenario(path) => Sim::from_scenario_at(&path).unwrap(),
            _ => unimplemented!(),
        };

        app.add_resource(OutcomeSimResource { sim })
            .add_system_to_stage(stage::UPDATE, trigger_sim_update.system());
    }
}

fn trigger_sim_update(mut sim_res: ResMut<OutcomeSimResource>) {
    println!("triggering sim update");
    sim_res
        .sim
        .event_queue
        .push(arraystring::new_unchecked("update"));
    sim_res.sim.step().unwrap();
}
