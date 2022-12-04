function handle_click() {
    let button = document.getElementById("survey-announcement-button");
    let timer = setInterval(function() {
        if (button) {
            clearInterval(timer);
            button = document.getElementById("survey-announcement-button");
            let survey_id = button.getAttribute("data-survey");
            button.addEventListener("click", function() {
                window.location.href = "/agenda#survey-" + survey_id;
                window.location.reload();
            });
        } else {
            button = document.getElementById("survey-announcement-button");
        }
    }, 200);
    
}
handle_click();
