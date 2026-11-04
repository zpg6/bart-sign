use sign_core::etd::{parse, Direction, Minutes};

#[test]
fn lafayette_yellow_line() {
    let json = include_bytes!("fixtures/lafy_leaving.json");
    let d = parse(json, Some("yellow")).unwrap();
    assert_eq!(d.time.as_str(), "07:12:32 PM PST");

    let north: Vec<_> = d.direction(Direction::North).iter().map(|g| g.destination.as_str()).collect();
    assert_eq!(north, ["Antioch", "Pittsburg/Bay Point"]);
    let antioch = &d.north[0];
    assert_eq!(antioch.trains[0].minutes, Minutes::Leaving);
    assert_eq!(antioch.trains[1].minutes, Minutes::In(19));
    assert_eq!(antioch.trains[2].cars, 10);

    let south: Vec<_> = d.direction(Direction::South).iter().map(|g| g.destination.as_str()).collect();
    assert_eq!(south, ["SF Airport", "SFO/Millbrae"]);
    assert_eq!(d.south[0].trains[0].minutes, Minutes::In(9));
    assert_eq!(d.south[0].trains[1].minutes, Minutes::In(17));
}

#[test]
fn montgomery_filters_by_line() {
    let json = include_bytes!("fixtures/mont_live.json");
    let all = parse(json, None).unwrap();
    assert_eq!(all.north.len() + all.south.len(), 8);

    let yellow = parse(json, Some("YELLOW")).unwrap();
    assert_eq!(yellow.north.len(), 1);
    assert_eq!(yellow.south.len(), 2);
    assert_eq!(yellow.south[0].destination.as_str(), "SF Airport");

    // Millbrae (red line) and SFO/Millbrae (yellow) share abbreviation MLBR
    // but must stay separate destinations.
    let south: Vec<_> = all.south.iter().map(|g| g.destination.as_str()).collect();
    assert!(south.contains(&"Millbrae") && south.contains(&"SFO/Millbrae"));
}

#[test]
fn lafayette_evening() {
    let d = parse(include_bytes!("fixtures/lafy_evening.json"), Some("yellow")).unwrap();
    // Groups are ordered by their soonest train.
    assert_eq!(d.north[0].destination.as_str(), "Pittsburg/Bay Point");
    assert_eq!(d.north[1].destination.as_str(), "Antioch");
    assert_eq!(d.north[1].trains[1].cars, 10);
}

#[test]
fn no_trains_and_garbage() {
    let empty = br#"{"?xml":{"@version":"1.0"},"root":{"@id":"1","date":"11/03/2026","time":"02:01:00 AM PST","station":[{"name":"Lafayette","abbr":"LAFY"}],"message":{"warning":"No data matched your criteria."}}}"#;
    let d = parse(empty, Some("yellow")).unwrap();
    assert!(d.north.is_empty() && d.south.is_empty());

    assert!(parse(b"<html>502</html>", None).is_err());
}

#[test]
fn cancelled_trains_dropped() {
    let json = br#"{"root":{"time":"x","station":[{"etd":[{"destination":"Antioch","abbreviation":"ANTC","estimate":[
        {"minutes":"4","direction":"North","length":"8","color":"YELLOW","cancelflag":"1"},
        {"minutes":"12","direction":"North","length":"8","color":"YELLOW","cancelflag":"0"}]}]}]}}"#;
    let d = parse(json, None).unwrap();
    assert_eq!(d.north[0].trains.len(), 1);
    assert_eq!(d.north[0].trains[0].minutes, Minutes::In(12));
}

#[test]
fn aging_counts_down_and_drops_departed_trains() {
    let d = parse(include_bytes!("fixtures/lafy_leaving.json"), Some("yellow")).unwrap();
    // Antioch: Leaving, 19, 39. SF Airport: 9, 17. SFO/Millbrae: 37, 57.
    let a = d.aged(10);
    let antioch = a.north.iter().find(|g| g.destination == "Antioch").unwrap();
    assert_eq!(antioch.trains[0].minutes, Minutes::In(9)); // Leaving dropped, 19 - 10
    let sfia = a.south.iter().find(|g| g.destination == "SF Airport").unwrap();
    assert_eq!(sfia.trains.len(), 1); // the 9 has gone
    assert_eq!(sfia.trains[0].minutes, Minutes::In(7));

    // Due exactly now: shown as 1 MIN, not 0.
    let a = d.aged(17);
    let sfia = a.south.iter().find(|g| g.destination == "SF Airport").unwrap();
    assert_eq!(sfia.trains[0].minutes, Minutes::In(1));

    // Hours later (network down): everything has departed.
    let a = d.aged(120);
    assert!(a.north.is_empty() && a.south.is_empty());
    assert_eq!(d.aged(0), d);
}
