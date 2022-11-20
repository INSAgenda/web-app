use crate::prelude::*;

pub fn t(s: &'static str) -> &'static str {
    if SETTINGS.lang() == Lang::French {
        return s;
    }
    
    match s {
        "Janvier" => "January",
        "Février" => "February",
        "Mars" => "March",
        "Avril" => "April",
        "Mai" => "May",
        "Juin" => "June",
        "Juillet" => "July",
        "Août" => "August",
        "Septembre" => "September",
        "Octobre" => "October",
        "Novembre" => "November",
        "Décembre" => "December",
        "Lundi" => "Monday",
        "Mardi" => "Tuesday",
        "Mercredi" => "Wednesday",
        "Jeudi" => "Thursday",
        "Vendredi" => "Friday",
        "Samedi" => "Saturday",
        "Dimanche" => "Sunday",
        
        s => {
            log!("Untranslated string: {}", s);
            s
        }
    }
}
