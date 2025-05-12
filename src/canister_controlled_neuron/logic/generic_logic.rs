use toolkit_utils::result::CanisterResult;

use crate::types::modules::{ModuleResponse, NeuronType};

use super::icp_neuron_logic::NeuronLogic;

pub struct GenericLogic;

impl GenericLogic {
    pub async fn tk_service_manage_neuron(module: NeuronType) -> CanisterResult<ModuleResponse> {
        match module {
            NeuronType::Icp(data) => NeuronLogic::handle_icp_neuron_args(data).await,
            NeuronType::Sns(_) => Ok(ModuleResponse::Boolean(true)),
        }
    }

    pub async fn tk_service_validate_manage_neuron(module: NeuronType) -> CanisterResult<String> {
        match module {
            NeuronType::Icp(data) => NeuronLogic::validate_icp_neuron_args(data).await,
            NeuronType::Sns(_) => Ok("".to_string()),
        }
    }
}
