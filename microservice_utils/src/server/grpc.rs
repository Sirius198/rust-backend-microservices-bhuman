use anyhow::*;
use uuid::Uuid;
use tonic::transport::Endpoint;

pub mod auth_service {
    tonic::include_proto!("auth_service");
}

pub mod user_service {
    tonic::include_proto!("user_service");
}

pub mod workspace_service {
    tonic::include_proto!("workspace_service");
}

use user_service::{user_service_client::UserServiceClient, AddWorkspaceRequest, RemoveWorkspaceRequest};
use workspace_service::{workspace_service_client::WorkspaceServiceClient, WorkspaceInfo};
use auth_service::{auth_service_client::AuthServiceClient, CheckTokenRequest, TokenRefreshRequest, CheckShopifyToken};


pub async fn add_workspace_id(user_id: &String, workspace_id: &Uuid) -> Result<()> {
    let endpoint: Endpoint = "http://localhost:4000".parse().context("Invalid endpoint")?;
    let mut grpc = UserServiceClient::connect(endpoint)
        .await
        .context("Unable to establish connection")?;
    let res = grpc
        .add_workspace_id(AddWorkspaceRequest {
            user_id: user_id.to_string(),
            workspace_id: workspace_id.to_string(),
        })
        .await
        .context("Unable to send echo request")?;

    println!("{:?}", res);

    Ok(())
}

pub async fn remove_workspace_id(user_id: &String, workspace_id: &Uuid) -> Result<()> {
    let endpoint: Endpoint = "http://localhost:4000".parse().context("Invalid endpoint")?;
    let mut grpc = UserServiceClient::connect(endpoint)
        .await
        .context("Unable to establish connection")?;
    let res = grpc
        .remove_workspace_id(RemoveWorkspaceRequest {
            user_id: user_id.to_string(),
            workspace_id: workspace_id.to_string(),
        })
        .await
        .context("Unable to send echo request")?;

    println!("{:?}", res);

    Ok(())
}

pub async fn check_token(user_id: &String, access_token: &String) -> Result<(), Error> {
    let endpoint: Endpoint = "http://localhost:4004".parse().context("Invalid endpoint")?;
    let mut grpc = AuthServiceClient::connect(endpoint)
        .await
        .context("Unable to establish connection")?;
    let res = grpc
        .check_token(CheckTokenRequest {
            user_id: user_id.to_string(),
            access_token: access_token.to_string(),
        })
        .await
        .context("Unable to send echo request")?;

    let message = res.into_inner();
    if message.status == "success" {
        println!("{:?}", message);
        Ok(())
    } else {
        Err(Error::msg("Authentication failed"))
    }
}

pub async fn refresh_token(user_id: &String, refresh_token: &Uuid) -> Result<(), Error> {
    let endpoint: Endpoint = "http://localhost:4004".parse().context("Invalid endpoint")?;
    let mut grpc = AuthServiceClient::connect(endpoint)
        .await
        .context("Unable to establish connection")?;
    let res = grpc
        .refresh_token(TokenRefreshRequest {
            user_id: user_id.to_string(),
            refresh_token: refresh_token.to_string(),
        })
        .await
        .context("Unable to send echo request")?;
   
    let message = res.into_inner();
    if message.status == "success" {
        println!("{:?}", message);
        Ok(())
    } else {
        Err(Error::msg("Authentication failed"))
    }
}

pub async fn get_shopify_token(user_id: &String) -> Result<String, Error> {
    let endpoint: Endpoint = "http://localhost:4004".parse().context("Invalid endpoint")?;
    let mut grpc = AuthServiceClient::connect(endpoint)
        .await
        .context("Unable to establish connection")?;
    let res = grpc
        .get_shopify_token(CheckShopifyToken {
            user_id: user_id.to_string(),
        })
        .await
        .context("Unable to send echo request")?;

    let message = res.into_inner();
    if message.status == "success" {
        println!("{:?}", message);
        Ok(message.token)
    } else {
        Err(Error::msg("Authentication failed"))
    }
}

pub async fn check_workspace(user_id: &String, workspace_id: &String) -> Result<(), Error> {
    let endpoint: Endpoint = "http://localhost:4001".parse().context("Invalid endpoint")?;
    let mut grpc = WorkspaceServiceClient::connect(endpoint)
        .await
        .context("Unable to establish connection")?;
    let res = grpc
        .check_workspace(WorkspaceInfo {
            user_id: user_id.to_string(),
            workspace_id: workspace_id.to_string(),
        })
        .await
        .context("Unable to send echo request")?;

    let message = res.into_inner();
    if message.status == "success" {
        println!("{:?}", message);
        Ok(())
    } else {
        Err(Error::msg("Workspace does not exist"))
    }
}