async function btn_dash_update(element_clicked, module, param) {
    let value = element_clicked.parentElement.children[0].value;
    let data = {'module': module, 'param': param, 'value': value};
    await post('/api/set_rc', data);
}
