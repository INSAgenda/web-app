const quotes = [
    {
        text: "We are all Satoshi.",
        author: "Satoshi"
    },
    {
        text: "Never gonna give you up. Never gonna let you down.",
        author: "Rick"
    },
    {
        text: "Soutenir que l'on ne se préoccupe pas de la vie privée car on n'a rien à cacher, c'est comme soutenir que l'on ne se préoccupe pas de la liberté d'expression parce qu'on n'a rien à dire.",
        author: "Edward Snowden"
    },
    {
        text: "Piètre disciple qui ne dépasse pas son maître.",
        author: "Léonard de Vinci"
    },
    {
        text: "FTX va bien.",
        author: "SBF"
    },
    {
        text: "The missile knows where it is at all times. It knows this because it knows where it isn't.",
        author: "1997 Air Force"
    }
];

const waiting_screen = document.getElementById("waiting-screen");
const waiting_screen_message = document.getElementById("waiting-screen-message");
const quote = document.getElementById("waiting-screen-quote");
const quote_text = document.getElementById("waiting-screen-quote-text");
const quote_author = document.getElementById("waiting-screen-quote-author");
const lds_ring = document.querySelector(".lds-ring");
const lds_bar = document.querySelector(".lds-bar");

function display_quote() {
    quote.style.opacity = "1";
    let quote_index = Math.floor(Math.random() * quotes.length);
    quote_text.innerText = quotes[quote_index].text;
    quote_author.innerText = quotes[quote_index].author;
    setTimeout(() => {
        quote.style.opacity = "0";
    }, 8600);
}

let interval = null;
export default function myInitializer () {
    return {
        onStart: () => {
            waiting_screen.style.opacity = "1";

            // Each 9 seconds, change the displayed quote
            setTimeout(() => {
                interval = setInterval(display_quote, 9000);
            }, 2000);
        },
        onProgress: ({current, total}) => {
            if (total === 0) {
                lds_ring.style.display = "block";
                lds_bar.style.display = "none";
            } else {
                lds_ring.style.display = "none";
                lds_bar.style.display = "block";
                lds_bar.querySelector("div").style.width = `${(current / total) * 100}%`;
            }
        },
        onComplete: () => {
            clearInterval(interval);
        },
        onSuccess: (wasm) => {
            lds_ring.style.display = "none";
            lds_bar.style.display = "none";
        },
        onFailure: (error) => {
            console.error("Failed to load the app", error);
            waiting_screen_message.innerText = "Quelque chose ne va pas...";
            setTimeout(() => { window.location.reload(); }, 700);
        }
    }
};
