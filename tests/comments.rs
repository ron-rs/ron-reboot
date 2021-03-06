use ron_reboot::print_error;

const INPUT: &str = r#"(
    // This is a UUID
    id: "14dec17f-ae14-40a3-8e44-e487fc423287",
    objects: [
        Entity(
            id: /* who would make a comment here? */ "62b3dbd1-56a8-469e-a262-41a66321da8b",
            components: [ // or here?
                (
                    type: "f5780013-bae4-49f0-ac0e-a108ff52fec0",
                    data: (
                        position: [100.0, 100.0]
                    ),
                ),
            ]
        ),
        /*
        Nested /* comments */ are fun as well!
        */
        ((
            id: "df6df3fd-4a0c-4640-bd71-7969f1e568a1",
            components: [
                (
                    type: "f5780013-bae4-49f0-ac0e-a108ff52fec0",
                    data: (
                        position: [200.0, 200.0]
                    ),
                ),
            ]
        )),
    ]
)"#;

#[test]
fn test_comments() {
    match ron_reboot::utf8_parser::ast_from_str(INPUT) {
        Ok(_) => {}
        Err(e) => {
            print_error(&e).unwrap();
            panic!();
        }
    }
}
