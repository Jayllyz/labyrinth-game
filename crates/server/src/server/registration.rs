//! Team and player registration.

use super::{GameServer, PlayerId, ServerResult};
use rand::{RngExt, rng};
use shared::messages::{Client, RegistrationError, Teams};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info};

impl GameServer {
    pub(super) fn register_client(&self, player: Client) -> ServerResult<PlayerId> {
        if player.player_name.is_empty() {
            return Err(RegistrationError::InvalidName);
        }

        if player.registration_token.is_empty() {
            return Err(RegistrationError::InvalidRegistrationToken);
        }

        let Ok(mut clients) = self.clients.lock() else {
            return Err(RegistrationError::ServerError);
        };

        let Ok(mut teams) = self.teams.lock() else {
            return Err(RegistrationError::ServerError);
        };

        // Every player needs a start cell other than the exit, and the board is set once
        // the game starts.
        if clients.len() + 1 >= self.config.width * self.config.height || self.has_started() {
            return Err(RegistrationError::TooManyPlayers);
        }

        let Some(team_name) = self.find_team_by_token(&teams, &player.registration_token) else {
            return Err(RegistrationError::InvalidRegistrationToken);
        };

        let id = PlayerId { team: team_name.clone(), name: player.player_name.clone() };
        if clients.contains_key(&id) {
            return Err(RegistrationError::AlreadyRegistered);
        }

        if let Some(team) = teams.get_mut(&team_name) {
            if team.players.len() >= self.config.max_players_per_team as usize {
                return Err(RegistrationError::TooManyPlayers);
            }

            team.players.push(player.clone());
            clients.insert(id.clone(), player.clone());

            info!("{} registered successfully on team {}", player.player_name, team_name);
            Ok(id)
        } else {
            error!("Team {team_name} not found");
            Err(RegistrationError::ServerError)
        }
    }

    pub(super) fn find_team_by_token(
        &self,
        teams: &HashMap<String, Teams>,
        token: &str,
    ) -> Option<String> {
        teams
            .iter()
            .find(|(_, team)| team.registration_token == token)
            .map(|(name, _)| name.clone())
    }

    pub(super) fn register_team(&self, mut team: Teams) -> ServerResult<String> {
        let mut teams = self.teams.lock().map_err(|_| RegistrationError::ServerError)?;

        if teams.contains_key(&team.team_name) {
            return Err(RegistrationError::TeamAlreadyRegistered);
        }
        if self.has_started() {
            return Err(RegistrationError::TooManyPlayers);
        }

        let token = self.generate_token();
        let team_name = team.team_name.clone();
        team.registration_token.clone_from(&token);

        teams.insert(team_name.clone(), team);

        info!("Team {team_name} registered successfully");
        Ok(token)
    }

    pub(super) fn generate_token(&self) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get timestamp")
            .as_secs()
            .to_string();
        let random_part: String =
            rng().sample_iter(&rand::distr::Alphanumeric).take(16).map(char::from).collect();

        format!("{timestamp}{random_part}")
    }
}
