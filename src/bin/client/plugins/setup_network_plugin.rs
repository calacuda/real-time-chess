use crate::client::{components::curent_client_id::CurrentClientId, systems::Connected};
use bevy::prelude::*;
use bevy_renet::{
    client_connected,
    netcode::{
        ClientAuthentication, NetcodeClientPlugin, NetcodeClientTransport, NetcodeTransportError,
    },
    renet::RenetClient,
};
use real_time_chess::{PROTOCOL_ID, connection_config};
use std::{net::UdpSocket, time::SystemTime};

/// If any error is found we just panic
#[allow(clippy::never_loop)]
fn panic_on_error_system(mut renet_error: EventReader<NetcodeTransportError>) {
    for e in renet_error.read() {
        error!("panicing with error: {}", e);
        panic!("{}", e);
    }
}

pub struct SetupNetwork;

impl Plugin for SetupNetwork {
    fn build(&self, app: &mut App) {
        let client = RenetClient::new(connection_config());

        let server_addr = "127.0.0.1:5000".parse().unwrap();
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let client_id = current_time.as_millis() as u64;
        let authentication = ClientAuthentication::Unsecure {
            client_id,
            protocol_id: PROTOCOL_ID,
            server_addr,
            user_data: None,
        };

        let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

        app.add_plugins(NetcodeClientPlugin)
            .configure_sets(Update, Connected.run_if(client_connected))
            .insert_resource(client)
            .insert_resource(transport)
            .insert_resource(CurrentClientId(client_id))
            .add_systems(Update, panic_on_error_system);
    }
}
