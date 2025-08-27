// selectors_to_replace: array of selectors to replace with the activity indicator
// state: true to enable, false to disable
function enable_activity_indicator(selectors_to_replace, state) {
    selectors_to_replace.forEach(selector => {
        document.querySelector(selector).style.display = state ? "none" : "";
    });
    if(state) {
        document.querySelector(selectors_to_replace[0]).insertAdjacentHTML('afterend', '<div class="lds-ring"><div></div><div></div><div></div><div></div></div>');
    } else {
        document.querySelector(".lds-ring").remove();
    }
}
