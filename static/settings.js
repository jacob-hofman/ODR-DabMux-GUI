async function btn_settings_add_service() {
    const template = document.getElementById('service_template');

    let clon = template.content.cloneNode(true);
    document.getElementById('services').appendChild(clon);
}

async function btn_settings_remove_service(element_clicked) {
    element_clicked.parentElement.remove()
}

async function btn_settings_send() {
    let data = {
        'instance_name': document.getElementById('instance_name').value,
        'dabmux_config_location': document.getElementById('dabmux_config_location').value,
        'tist': document.getElementById('tist').checked,
        'tist_offset': parseInt(document.getElementById('tist_offset').value, 10),
        'ensemble_id': parseInt(document.getElementById('ensemble_id').value, 16),
        'ensemble_ecc': parseInt(document.getElementById('ensemble_ecc').value, 16),
        'ensemble_label': document.getElementById('ensemble_label').value,
        'ensemble_shortlabel': document.getElementById('ensemble_shortlabel').value,
        'output_edi_port': parseInt(document.getElementById('output_edi_port').value, 10),
        'services': [],
    };

    const services = document.getElementById('services');
    const destList = services.querySelectorAll("p.service");
    for (let i = 0; i < destList.length; i++) {
        data.services.push({
            'unique_id': destList[i].querySelector("input.srv_unique_id").value,
            'sid': parseInt(destList[i].querySelector("input.srv_sid").value, 16),
            'ecc': parseInt(destList[i].querySelector("input.srv_ecc").value, 16),
            'label': destList[i].querySelector("input.srv_label").value,
            'shortlabel': destList[i].querySelector("input.srv_shortlabel").value,
            'input_port': parseInt(destList[i].querySelector("input.srv_input_port").value, 10),
            'bitrate': parseInt(destList[i].querySelector("input.srv_bitrate").value, 10),
            'protection': parseInt(destList[i].querySelector("input.srv_protection").value, 10),
        });
    }

    await post('/api/settings', data);
}

