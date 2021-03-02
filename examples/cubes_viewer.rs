#![allow(unused)]

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, PrintDiagnosticsPlugin};
use bevy::prelude::*;

use bevy::pbr::render_graph::FORWARD_PIPELINE_HANDLE;
use bevy::render::pipeline::RenderPipeline;
use bevy::render::render_graph::base::MainPass;

use bevy_outcome::{
    OutcomeClientPlugin, OutcomeClientResource, OutcomeSimPlugin, OutcomeWorkerPlugin, SyncMarker,
};
use outcome_core::Address;

use outcome_net::msg::DataTransferResponse;
use outcome_net::msg::TransferResponseData;

fn main() {
    App::build()
        .add_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        // .add_plugins(MinimalPlugins)
        .add_plugin(OutcomeClientPlugin::with_server_addr("127.0.0.1:9123"))
        .add_resource(AppState::default())
        .add_system_to_stage(stage::UPDATE, cube_sync.system())
        // .add_system_to_stage(stage::UPDATE, spawn_missing_entities.system())
        // .add_plugin(OutcomeSimPlugin::with_scenario(
        //     std::env::args().nth(1).unwrap_or("./".to_string()).as_str(),
        // ))
        // .add_resource(AppState::default())
        .add_startup_system(setup_scene.system())
        // .add_startup_system(sim_sync.system())
        // FPS counter (console)
        // .add_plugin(PrintDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // .add_system(PrintDiagnosticsPlugin::print_diagnostics_system.system())
        .run();
}

#[derive(Default)]
struct AppState {
    cube_mesh: Option<Handle<Mesh>>,
    cube_material: Option<Handle<StandardMaterial>>,
}

/// A component bundle for a synced cube
#[derive(Bundle)]
pub struct CubeBundle {
    pub sync_marker: SyncMarker,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub main_pass: bevy::render::render_graph::base::MainPass,
    pub draw: Draw,
    pub visible: Visible,
    pub render_pipelines: RenderPipelines,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for CubeBundle {
    fn default() -> Self {
        Self {
            sync_marker: SyncMarker(None),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                FORWARD_PIPELINE_HANDLE.typed(),
            )]),
            mesh: Default::default(),
            visible: Default::default(),
            material: Default::default(),
            main_pass: Default::default(),
            draw: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
}

fn cube_sync(
    cmds: &mut Commands,
    //mut resources: &mut Resources,
    mut app_state: ResMut<AppState>,
    mut client_res: ResMut<OutcomeClientResource>,
    mut query: Query<(&SyncMarker, &mut Transform)>,
) {
    //let client = &mut resources
    //.get_thread_local_mut::<OutcomeClientResource>()
    //.unwrap()
    //.client;

    let client = &mut client_res.client.lock().unwrap();

    //if !client.{
    //return;
    //}

    if let Ok(msg) = client.connection.try_recv_msg() {
        match msg.type_ {
            outcome_net::msg::MessageType::DataTransferResponse => {
                if let Ok(resp) =
                    msg.unpack_payload::<DataTransferResponse>(client.connection.encoding())
                {
                    //let resp: DataTransferResponse =
                    //msg.unpack_payload(client.connection.encoding()).unwrap();

                    match resp.data {
                        Some(TransferResponseData::VarOrdered(order_id, data)) => {
                            for (n, var) in data.vars.iter().enumerate() {
                                if n == 0 || n == 1 {
                                    continue;
                                }
                                //let addr = Address::from_str(&var_addr[1..]).unwrap();
                                //let ent_id = addr.entity.as_str().parse::<u32>().unwrap();
                                // for (sync, pbf) in q.iter_mut() {
                                //     println!("cube syncmarker ent_id: {}", sync.0);
                                // }
                                if let Some((sync, mut transform)) =
                                    query.iter_mut().find(|(s, _)| s.0 == Some(n as u32))
                                {
                                    // entity exists, update the value
                                    transform.translation.x = *var.as_float().unwrap() as f32;
                                } else {
                                    // entity doesn't exist, spawn it
                                    // app_state.to_spawn.push(ent_id);
                                    println!("spawning new cube, ent_id: {}", n);
                                    // cmds.spawn((
                                    //     SyncMarker(ent_id),
                                    //     PbrBundle {
                                    //         mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                                    //         // mesh: mesh_handle.clone(),
                                    //         material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                                    //         transform: Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
                                    //         ..Default::default()
                                    //     },
                                    // ));

                                    cmds.spawn(CubeBundle {
                                        sync_marker: SyncMarker(Some(n as u32)),
                                        mesh: app_state.cube_mesh.as_ref().unwrap().clone(),
                                        material: app_state.cube_material.as_ref().unwrap().clone(),
                                        transform: Transform::from_translation(Vec3::new(
                                            0.0, 0.5, 0.0,
                                        )),
                                        ..Default::default()
                                    });
                                }
                            }
                        }
                        Some(TransferResponseData::Typed(data)) => {
                            for (var_addr, val) in data.floats {
                                let addr = Address::from_str(&var_addr[1..]).unwrap();
                                let ent_id = addr.entity.as_str().parse::<u32>().unwrap();
                                // for (sync, pbf) in q.iter_mut() {
                                //     println!("cube syncmarker ent_id: {}", sync.0);
                                // }
                                if let Some((sync, mut transform)) =
                                    query.iter_mut().find(|(s, _)| s.0 == Some(ent_id))
                                {
                                    // entity exists, update the value
                                    match addr.var_id.as_str() {
                                        "x" => transform.translation.x = val as f32,
                                        "y" => transform.translation.y = val as f32,
                                        "z" => transform.translation.z = val as f32,
                                        _ => (),
                                    }
                                } else {
                                    // entity doesn't exist, spawn it
                                    // app_state.to_spawn.push(ent_id);
                                    println!("spawning new cube, ent_id: {}", ent_id);
                                    // cmds.spawn((
                                    //     SyncMarker(ent_id),
                                    //     PbrBundle {
                                    //         mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                                    //         // mesh: mesh_handle.clone(),
                                    //         material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                                    //         transform: Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
                                    //         ..Default::default()
                                    //     },
                                    // ));

                                    cmds.spawn(CubeBundle {
                                        sync_marker: SyncMarker(Some(ent_id)),
                                        mesh: app_state.cube_mesh.as_ref().unwrap().clone(),
                                        material: app_state.cube_material.as_ref().unwrap().clone(),
                                        transform: Transform::from_translation(Vec3::new(
                                            0.0, 0.5, 0.0,
                                        )),
                                        ..Default::default()
                                    });
                                }
                            }
                        }
                        _ => {
                            //println!("{:?}", resp);
                            //continue;
                        }
                    }
                } else {
                    println!("failed deserializing datatransferresponse {:?}", msg.type_);
                    //break;
                }
            }
            _ => println!("{:?}", msg.type_),
        }
    }

    while let Ok(event) = client.connection.try_recv() {
        //println!("event: {:?}", event);
    }

    //if let Err(e) = client.connection.try_recv_msg() {
    //println!("{:?}", e);
    //}
    //let trd = client.lock().unwrap().get_vars().unwrap();

    //client.lock().unwrap().server_step_request(1).unwrap();
}

/// set up a simple 3D scene
fn setup_scene(
    commands: &mut Commands,
    mut client_res: ResMut<OutcomeClientResource>,
    mut app_state: ResMut<AppState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    client_res
        .client
        .lock()
        .unwrap()
        .reg_scheduled_transfer()
        .unwrap();

    let mesh_handle = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let material_handle = materials.add(Color::rgb(0.8, 0.7, 0.6).into());
    app_state.cube_mesh = Some(mesh_handle.clone());
    app_state.cube_material = Some(material_handle.clone());
    // add entities to the world
    commands
        // plane
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 100.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform: Transform::from_translation(Vec3::new(0.0, -10.0, 0.0)),
            ..Default::default()
        })
        // cube
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_translation(Vec3::new(20.0, 0.5, 0.0)),
            ..Default::default()
        })
        // light
        .spawn(LightBundle {
            transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
            ..Default::default()
        })
        // camera
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(100.0, 100.0, 5.0))
                .looking_at(Vec3::default(), Vec3::unit_y()),
            ..Default::default()
        });
}
