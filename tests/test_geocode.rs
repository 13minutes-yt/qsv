use crate::workdir::Workdir;

#[test]
fn geocode_suggest() {
    let wrk = Workdir::new("geocode_suggest");
    wrk.create(
        "data.csv",
        vec![
            svec!["Location"],
            svec!["Melrose, New York"],
            svec!["East Flatbush, New York"],
            svec!["Manhattan, New York"],
            svec!["Brooklyn, New York"],
            svec!["East Harlem, New York"],
            svec!["This is not a Location and it will not be geocoded"],
            svec!["Jersey City, New Jersey"],
            svec!["95.213424, 190,1234565"], // invalid lat, long
            svec!["Makati, Metro Manila, Philippines"],
        ],
    );
    let mut cmd = wrk.command("geocode");
    cmd.arg("suggest").arg("Location").arg("data.csv");

    let got: Vec<Vec<String>> = wrk.read_stdout(&mut cmd);
    let expected = vec![
        svec!["Location"],
        svec!["(41.90059, -87.85673)"],
        svec!["(40.65371, -73.93042)"],
        svec!["(40.71427, -74.00597)"],
        svec!["(45.09413, -93.35634)"],
        svec!["(40.79472, -73.9425)"],
        svec!["This is not a Location and it will not be geocoded"],
        svec!["(40.72816, -74.07764)"],
        svec!["95.213424, 190,1234565"], // suggest expects a city name, not lat, long
        svec!["(14.55027, 121.03269)"],
    ];
    assert_eq!(got, expected);
}

#[test]
fn geocode_suggest_intl() {
    let wrk = Workdir::new("geocode_suggest_intl");
    wrk.create(
        "data.csv",
        vec![
            svec!["Location"],
            svec!["Paris"],
            svec!["Manila"],
            svec!["London"],
            svec!["Berlin"],
            svec!["Moscow"],
            svec!["This is not a Location and it will not be geocoded"],
            svec!["Brazil"],
            svec!["95.213424, 190,1234565"], // invalid lat, long
            svec!["Havana"],
        ],
    );
    let mut cmd = wrk.command("geocode");
    cmd.arg("suggest")
        .arg("Location")
        .args(["-f", "%city-admin1-country"])
        .arg("data.csv");

    let got: Vec<Vec<String>> = wrk.read_stdout(&mut cmd);
    let expected = vec![
        svec!["Location"],
        svec!["Paris, Île-de-France Region France"],
        svec!["Manila, National Capital Region Philippines"],
        svec!["London, England United Kingdom"],
        svec!["Berlin,  Germany"],
        svec!["Moscow, Moscow Russia"],
        svec!["This is not a Location and it will not be geocoded"],
        svec!["Brasília, Federal District Brazil"],
        svec!["95.213424, 190,1234565"],
        svec!["Havana, La Habana Province Cuba"],
    ];
    assert_eq!(got, expected);
}

#[test]
fn geocode_suggest_intl_country_filter() {
    let wrk = Workdir::new("geocode_suggest_intl_country_filter");
    wrk.create(
        "data.csv",
        vec![
            svec!["Location"],
            svec!["Paris"],
            svec!["Manila"],
            svec!["London"],
            svec!["Berlin"],
            svec!["Moscow"],
            svec!["This is not a Location and it will not be geocoded"],
            svec!["Brazil"],
            svec!["95.213424, 190,1234565"], // invalid lat, long
            svec!["Havana"],
        ],
    );
    let mut cmd = wrk.command("geocode");
    cmd.arg("suggest")
        .arg("Location")
        .args(["--country", "us"])
        .args(["-f", "%city-admin1-country"])
        .arg("data.csv");

    let got: Vec<Vec<String>> = wrk.read_stdout(&mut cmd);
    let expected = vec![
        svec!["Location"],
        svec!["Paris, Texas United States"],
        svec!["Manteca, California United States"],
        svec!["Sterling, Virginia United States"],
        svec!["Burlington, North Carolina United States"],
        svec!["Moscow, Idaho United States"],
        svec!["This is not a Location and it will not be geocoded"],
        svec!["Bradley, Illinois United States"],
        svec!["95.213424, 190,1234565"],
        svec!["Savannah, Georgia United States"],
    ];
    assert_eq!(got, expected);
}

#[test]
fn geocode_suggest_intl_multi_country_filter() {
    let wrk = Workdir::new("geocode_suggest_intl_multi_country_filter");
    wrk.create(
        "data.csv",
        vec![
            svec!["Location"],
            svec!["Paris"],
            svec!["Manila"],
            svec!["London"],
            svec!["Berlin"],
            svec!["Moscow"],
            svec!["This is not a Location and it will not be geocoded"],
            svec!["Brazil"],
            svec!["95.213424, 190,1234565"], // invalid lat, long
            svec!["Havana"],
        ],
    );
    let mut cmd = wrk.command("geocode");
    cmd.arg("suggest")
        .arg("Location")
        .args(["--country", "us,fr,ru"])
        .args(["-f", "%city-admin1-country"])
        .arg("data.csv");

    let got: Vec<Vec<String>> = wrk.read_stdout(&mut cmd);
    let expected = vec![
        svec!["Location"],
        svec!["Paris, Île-de-France Region France"],
        svec!["Manteca, California United States"],
        svec!["Sterling, Virginia United States"],
        svec!["Burlington, North Carolina United States"],
        svec!["Moscow, Moscow Russia"],
        svec!["This is not a Location and it will not be geocoded"],
        svec!["Bradley, Illinois United States"],
        svec!["95.213424, 190,1234565"],
        svec!["Savannah, Georgia United States"],
    ];
    assert_eq!(got, expected);
}

#[test]
fn geocode_suggest_invalid() {
    let wrk = Workdir::new("geocode_suggest_invalid");
    wrk.create(
        "data.csv",
        vec![
            svec!["Location"],
            svec!["Melrose, New York"],
            svec!["East Flatbush, New York"],
            svec!["Manhattan, New York"],
            svec!["East Harlem, New York"],
            svec!["Brooklyn, New York"],
            svec!["This is not a Location and it will not be geocoded"],
            svec!["Jersey City, New Jersey"],
            svec!["95.213424, 190,1234565"], // invalid lat, long
            svec!["Makati, Metro Manila, Philippines"],
        ],
    );
    let mut cmd = wrk.command("geocode");
    cmd.arg("suggest")
        .arg("Location")
        .args(["--invalid-result", "<ERROR>"])
        .arg("data.csv");

    let got: Vec<Vec<String>> = wrk.read_stdout(&mut cmd);
    let expected = vec![
        svec!["Location"],
        svec!["(41.90059, -87.85673)"],
        svec!["(40.65371, -73.93042)"],
        svec!["(40.71427, -74.00597)"],
        svec!["(40.79472, -73.9425)"],
        svec!["(45.09413, -93.35634)"],
        svec!["<ERROR>"],
        svec!["(40.72816, -74.07764)"],
        svec!["<ERROR>"], // suggest expects a city name, not lat, long
        svec!["(14.55027, 121.03269)"],
    ];
    assert_eq!(got, expected);
}

#[test]
fn geocode_suggest_dynfmt() {
    let wrk = Workdir::new("geocode_suggest_dynfmt");
    wrk.create(
        "data.csv",
        vec![
            svec!["Location"],
            svec!["Melrose, New York"],
            svec!["East Flatbush, New York"],
            svec!["Manhattan, New York"],
            svec!["East Harlem, New York"],
            svec!["This is not a Location and it will not be geocoded"],
            svec!["95.213424, 190,1234565"], // invalid lat, long
            svec!["Makati, Metro Manila, Philippines"],
        ],
    );
    let mut cmd = wrk.command("geocode");
    cmd.arg("suggest")
        .arg("Location")
        .arg("--formatstr")
        .arg("{latitude}:{longitude} - {name}, {admin1}")
        .arg("data.csv");

    let got: Vec<Vec<String>> = wrk.read_stdout(&mut cmd);
    let expected = vec![
        svec!["Location"],
        svec!["41.90059:-87.85673 - Melrose Park, Illinois"],
        svec!["40.65371:-73.93042 - East Flatbush, New York"],
        svec!["40.71427:-74.00597 - New York City, New York"],
        svec!["40.79472:-73.9425 - East Harlem, New York"],
        svec!["This is not a Location and it will not be geocoded"],
        svec!["95.213424, 190,1234565"], // invalid lat, long
        svec!["14.55027:121.03269 - Makati City, National Capital Region"],
    ];
    assert_eq!(got, expected);
}

#[test]
fn geocode_suggest_invalid_dynfmt() {
    let wrk = Workdir::new("geocode_suggest_invalid_dynfmt");
    wrk.create(
        "data.csv",
        vec![
            svec!["Location"],
            svec!["Melrose, New York"],
            svec!["East Flatbush, New York"],
            svec!["This is not a Location and it will not be geocoded"],
            svec!["95.213424, 190,1234565"], // invalid lat, long
            svec!["Makati, Metro Manila, Philippines"],
        ],
    );
    let mut cmd = wrk.command("geocode");
    cmd.arg("suggest")
        .arg("Location")
        .arg("--formatstr")
        .arg("{latitude}:{longitude} - {name}, {admin1} {invalid_field}")
        .arg("data.csv");

    let got: Vec<Vec<String>> = wrk.read_stdout(&mut cmd);
    let expected = vec![
        svec!["Location"],
        svec!["Invalid dynfmt template."],
        svec!["Invalid dynfmt template."],
        svec!["This is not a Location and it will not be geocoded"],
        svec!["95.213424, 190,1234565"], // invalid lat, long
        svec!["Invalid dynfmt template."],
    ];
    assert_eq!(got, expected);
}

#[test]
fn geocode_suggest_fmt() {
    let wrk = Workdir::new("geocode_suggest_fmt");
    wrk.create(
        "data.csv",
        vec![
            svec!["Location"],
            svec!["Elmhurst, New York"],
            svec!["East Flatbush, New York"],
            svec!["Manhattan, New York"],
            svec!["East Harlem, New York"],
            svec!["This is not a Location and it will not be geocoded"],
            svec!["40.71427, -74.00597"],
            svec!["Makati, Metro Manila, Philippines"],
        ],
    );
    let mut cmd = wrk.command("geocode");
    cmd.arg("suggest")
        .arg("Location")
        .arg("--formatstr")
        .arg("%city-state-country")
        .arg("data.csv");

    let got: Vec<Vec<String>> = wrk.read_stdout(&mut cmd);
    let expected = vec![
        svec!["Location"],
        svec!["Elmhurst, New York United States"],
        svec!["East Flatbush, New York United States"],
        svec!["New York City, New York United States"],
        svec!["East Harlem, New York United States"],
        svec!["This is not a Location and it will not be geocoded"],
        svec!["40.71427, -74.00597"], // suggest doesn't work with lat, long
        svec!["Makati City, National Capital Region Philippines"],
    ];
    assert_eq!(got, expected);
}

#[test]
fn geocode_suggest_fmt_cityrecord() {
    let wrk = Workdir::new("geocode_suggest_fmt_cityrecord");
    wrk.create(
        "data.csv",
        vec![
            svec!["Location"],
            svec!["Elmhurst, New York"],
            svec!["East Flatbush, New York"],
            svec!["Manhattan, New York"],
            svec!["East Harlem, New York"],
            svec!["This is not a Location and it will not be geocoded"],
            svec!["40.71427, -74.00597"],
            svec!["Makati, Metro Manila, Philippines"],
        ],
    );
    let mut cmd = wrk.command("geocode");
    cmd.arg("suggest")
        .arg("Location")
        .arg("--formatstr")
        .arg("%cityrecord")
        .arg("data.csv");

    let got: Vec<Vec<String>> = wrk.read_stdout(&mut cmd);
    let expected = vec![
        svec!["Location"],
        svec![
            "CitiesRecord { id: 5116495, name: \"Elmhurst\", latitude: 40.73649, longitude: \
             -73.87791, country: Some(Country { id: 6252001, code: \"US\", name: \"United \
             States\" }), admin_division: Some(AdminDivision { id: 5128638, code: \"US.NY\", \
             name: \"New York\" }), admin2_division: Some(AdminDivision { id: 5133268, code: \
             \"US.NY.081\", name: \"Queens County\" }), timezone: \"America/New_York\", names: \
             Some({\"en\": \"Elmhurst\"}), country_names: Some({\"en\": \"United States\"}), \
             admin1_names: Some({\"en\": \"New York\"}), admin2_names: Some({\"en\": \"Queens \
             County\"}), population: 113364 }"
        ],
        svec![
            "CitiesRecord { id: 5115843, name: \"East Flatbush\", latitude: 40.65371, longitude: \
             -73.93042, country: Some(Country { id: 6252001, code: \"US\", name: \"United \
             States\" }), admin_division: Some(AdminDivision { id: 5128638, code: \"US.NY\", \
             name: \"New York\" }), admin2_division: Some(AdminDivision { id: 6941775, code: \
             \"US.NY.047\", name: \"Kings County\" }), timezone: \"America/New_York\", names: \
             Some({\"en\": \"East Flatbush\"}), country_names: Some({\"en\": \"United States\"}), \
             admin1_names: Some({\"en\": \"New York\"}), admin2_names: Some({\"en\": \"Kings\"}), \
             population: 178464 }"
        ],
        svec![
            "CitiesRecord { id: 5128581, name: \"New York City\", latitude: 40.71427, longitude: \
             -74.00597, country: Some(Country { id: 6252001, code: \"US\", name: \"United \
             States\" }), admin_division: Some(AdminDivision { id: 5128638, code: \"US.NY\", \
             name: \"New York\" }), admin2_division: None, timezone: \"America/New_York\", names: \
             Some({\"en\": \"New York\"}), country_names: Some({\"en\": \"United States\"}), \
             admin1_names: Some({\"en\": \"New York\"}), admin2_names: None, population: 8804190 }"
        ],
        svec![
            "CitiesRecord { id: 6332428, name: \"East Harlem\", latitude: 40.79472, longitude: \
             -73.9425, country: Some(Country { id: 6252001, code: \"US\", name: \"United States\" \
             }), admin_division: Some(AdminDivision { id: 5128638, code: \"US.NY\", name: \"New \
             York\" }), admin2_division: Some(AdminDivision { id: 5128594, code: \"US.NY.061\", \
             name: \"New York County\" }), timezone: \"America/New_York\", names: None, \
             country_names: Some({\"en\": \"United States\"}), admin1_names: Some({\"en\": \"New \
             York\"}), admin2_names: Some({\"en\": \"New York County\"}), population: 115921 }"
        ],
        svec!["This is not a Location and it will not be geocoded"],
        svec!["40.71427, -74.00597"],
        svec![
            "CitiesRecord { id: 1703417, name: \"Makati City\", latitude: 14.55027, longitude: \
             121.03269, country: Some(Country { id: 1694008, code: \"PH\", name: \"Philippines\" \
             }), admin_division: Some(AdminDivision { id: 7521311, code: \"PH.NCR\", name: \
             \"Metro Manila\" }), admin2_division: Some(AdminDivision { id: 11395838, code: \
             \"PH.NCR.137600000\", name: \"Southern Manila District\" }), timezone: \
             \"Asia/Manila\", names: Some({\"en\": \"Makati City\"}), country_names: \
             Some({\"en\": \"Philippines\"}), admin1_names: Some({\"en\": \"National Capital \
             Region\"}), admin2_names: None, population: 510383 }"
        ],
    ];
    assert_eq!(got, expected);
}

#[test]
fn geocode_reverse() {
    let wrk = Workdir::new("geocode_reverse");
    wrk.create(
        "data.csv",
        vec![
            svec!["Location"],
            svec!["40.812126, -73.9041813"],
            svec!["40.66472342, -73.93867227"],
            svec!["(40.766672, -73.9568128)"],
            svec!["(  40.819342, -73.9532127    )"],
            svec!["< 40.819342,-73.9532127 >"],
            svec!["This is not a Location and it will not be geocoded"],
            svec![
                "The treasure is at these coordinates 40.66472342, -73.93867227. This should be \
                 geocoded."
            ],
            svec!["95.213424, 190,1234565"], // invalid lat, long
            svec![
                "The coordinates are 40.66472342 latitude, -73.93867227 longitudue. This should \
                 NOT be geocoded."
            ],
        ],
    );
    let mut cmd = wrk.command("geocode");
    cmd.arg("reverse").arg("Location").arg("data.csv");

    let got: Vec<Vec<String>> = wrk.read_stdout(&mut cmd);
    let expected = vec![
        svec!["Location"],
        svec!["Melrose, New York"],
        svec!["East Flatbush, New York"],
        svec!["Manhattan, New York"],
        svec!["East Harlem, New York"],
        svec!["East Harlem, New York"],
        svec!["This is not a Location and it will not be geocoded"],
        svec!["East Flatbush, New York"],
        svec!["95.213424, 190,1234565"], // invalid lat, long
        svec![
            "The coordinates are 40.66472342 latitude, -73.93867227 longitudue. This should NOT \
             be geocoded."
        ],
    ];
    assert_eq!(got, expected);
}

#[test]
fn geocode_reverse_fmtstring() {
    let wrk = Workdir::new("geocode_reverse_fmtstring");
    wrk.create(
        "data.csv",
        vec![
            svec!["Location"],
            svec!["40.812126, -73.9041813"],
            svec!["40.66472342, -73.93867227"],
            svec!["(40.766672, -73.9568128)"],
            svec!["(40.819342, -73.9532127)"],
            svec!["This is not a Location and it will not be geocoded"],
            svec!["95.213424, 190,1234565"], // invalid lat,long
        ],
    );
    let mut cmd = wrk.command("geocode");
    cmd.arg("reverse")
        .arg("Location")
        .arg("--formatstr")
        .arg("%city-state-country")
        .arg("data.csv");

    let got: Vec<Vec<String>> = wrk.read_stdout(&mut cmd);
    let expected = vec![
        svec!["Location"],
        svec!["Melrose, New York United States"],
        svec!["East Flatbush, New York United States"],
        svec!["Manhattan, New York United States"],
        svec!["East Harlem, New York United States"],
        svec!["This is not a Location and it will not be geocoded"],
        svec!["95.213424, 190,1234565"], // invalid lat,long
    ];
    assert_eq!(got, expected);
}

#[test]
fn geocode_reverse_fmtstring_intl() {
    let wrk = Workdir::new("geocode_reverse_fmtstring_intl");
    wrk.create(
        "data.csv",
        vec![
            svec!["Location"],
            svec!["41.390205, 2.154007"],
            svec!["52.371807, 4.896029"],
            svec!["(52.520008, 13.404954)"],
            svec!["(14.55027,121.03269)"],
            svec!["This is not a Location and it will not be geocoded"],
            svec!["95.213424, 190,1234565"], // invalid lat,long
        ],
    );
    let mut cmd = wrk.command("geocode");
    cmd.arg("reverse")
        .arg("Location")
        .arg("--formatstr")
        .arg("%city-admin1-country")
        .arg("data.csv");

    let got: Vec<Vec<String>> = wrk.read_stdout(&mut cmd);
    let expected = vec![
        svec!["Location"],
        svec!["Barcelona, Catalonia Spain"],
        svec!["Amsterdam, North Holland Netherlands"],
        svec!["Berlin,  Germany"],
        svec!["Makati City, National Capital Region Philippines"],
        svec!["This is not a Location and it will not be geocoded"],
        svec!["95.213424, 190,1234565"], // invalid lat,long
    ];
    assert_eq!(got, expected);
}

#[test]
fn geocode_reverse_fmtstring_intl_dynfmt() {
    let wrk = Workdir::new("geocode_reverse_fmtstring_intl_dynfmt");
    wrk.create(
        "data.csv",
        vec![
            svec!["Location"],
            svec!["41.390205, 2.154007"],
            svec!["52.371807, 4.896029"],
            svec!["(52.520008, 13.404954)"],
            svec!["(14.55027,121.03269)"],
            svec!["This is not a Location and it will not be geocoded"],
            svec!["95.213424, 190,1234565"], // invalid lat,long
        ],
    );
    let mut cmd = wrk.command("geocode");
    cmd.arg("reverse")
        .arg("Location")
        .arg("--formatstr")
        .arg("pop: {population} tz: {timezone}")
        .arg("data.csv");

    let got: Vec<Vec<String>> = wrk.read_stdout(&mut cmd);
    let expected = vec![
        svec!["Location"],
        svec!["pop: 1620343 tz: Europe/Madrid"],
        svec!["pop: 741636 tz: Europe/Amsterdam"],
        svec!["pop: 3426354 tz: Europe/Berlin"],
        svec!["pop: 510383 tz: Asia/Manila"],
        svec!["This is not a Location and it will not be geocoded"],
        svec!["95.213424, 190,1234565"], // invalid lat,long
    ];
    assert_eq!(got, expected);
}

#[test]
fn geocode_reverse_fmtstring_intl_invalid_dynfmt() {
    let wrk = Workdir::new("geocode_reverse_fmtstring_intl_invalid_dynfmt");
    wrk.create(
        "data.csv",
        vec![
            svec!["Location"],
            svec!["41.390205, 2.154007"],
            svec!["52.371807, 4.896029"],
            svec!["(52.520008, 13.404954)"],
            svec!["(14.55027,121.03269)"],
            svec!["This is not a Location and it will not be geocoded"],
            svec!["95.213424, 190,1234565"], // invalid lat,long
        ],
    );
    let mut cmd = wrk.command("geocode");
    cmd.arg("reverse")
        .arg("Location")
        .arg("--formatstr")
        .arg("pop: {population} tz: {timezone} {doesnotexistfield}")
        .arg("data.csv");

    let got: Vec<Vec<String>> = wrk.read_stdout(&mut cmd);
    let expected = vec![
        svec!["Location"],
        svec!["Invalid dynfmt template."],
        svec!["Invalid dynfmt template."],
        svec!["Invalid dynfmt template."],
        svec!["Invalid dynfmt template."],
        svec!["This is not a Location and it will not be geocoded"],
        svec!["95.213424, 190,1234565"], // invalid lat,long
    ];
    assert_eq!(got, expected);
}
