function autocomplete_email_el(e) {
    if (e.inputType == "insertText" && email.value.endsWith("@") && (email.value.split("@").length - 1) == 1) {
        email.disabled = true;
        let current_value = email.value;
        let incoming_chars = "insa-rouen.fr";
        let i = 0;
        let interval = setInterval(function() {
            current_value = current_value + incoming_chars[i];
            email.value = current_value;
            i++;
            if (i == incoming_chars.length) {
                clearInterval(interval); 
                setTimeout(function() {
                    focus_next();
                    email.disabled = false;
                }, 500);
            }
        }, 15);
    }
}
