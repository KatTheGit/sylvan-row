/// Common functions and structs used by both client and server.
pub mod common;
/// Functions related to drawing the user interface.
pub mod ui;
/// Functions and structs related to any form of maths
/// or logic, like `Vector2` or movement logic functions.
pub mod maths;
/// Constant parameters, like TILE_SIZE, DEFAULT_IP_ADDRESS, etc...
pub mod const_params;
/// Gameobjects, Character Properties, any data that expresses anything regarding
/// the game.
pub mod gamedata;
/// Functions and structs related to drawing things to the screen
pub mod graphics;
/// Game server. Called by mothership when creating an instance for players.
pub mod gameserver;
/// Structs that need to be shared between the client and the mothership server.
pub mod mothership_common;
/// Interface for the server's database.
pub mod database;
/// Filters for the chat, user registration, etc...
pub mod filter;
/// Netcode
pub mod network;
/// Abstraction for playing sounds & stuff.
pub mod audio;