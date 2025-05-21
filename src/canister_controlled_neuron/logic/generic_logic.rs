use toolkit_utils::result::CanisterResult;

use crate::types::modules::{ModuleResponse, NeuronType};

use super::{icp_neuron_logic::ICPNeuronLogic, sns_neuron_logic::SNSNeuronLogic};

pub struct GenericLogic;

impl GenericLogic {
    pub async fn tk_service_manage_neuron(module: NeuronType) -> CanisterResult<ModuleResponse> {
        match module {
            NeuronType::Icp(data) => ICPNeuronLogic::handle_icp_neuron_args(data).await,
            NeuronType::Sns(data) => SNSNeuronLogic::handle_sns_neuron_args(data).await,
        }
    }

    pub async fn tk_service_validate_manage_neuron(module: NeuronType) -> CanisterResult<String> {
        match module {
            NeuronType::Icp(data) => ICPNeuronLogic::validate_icp_neuron_args(data).await,
            NeuronType::Sns(_) => Ok("".to_string()),
        }
    }
}
