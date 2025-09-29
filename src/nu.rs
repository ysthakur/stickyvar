//! Setup code for Nushell

pub fn init(sv_path: &str) -> String {
    format!(r#"export module sv {{
    # Set a variable, both as an environment variable and in the sticky variable database
    export def --env set [name: string, value: string] {{
        load-env {{ $name: $value }}
        {sv_path} set $name $value
    }}

    # If given a variable name, load that variable. Otherwise, load all variables
    # in database.
    export def --env load [ name?: string ] {{
        if $name != (null) {{
            let value = {sv_path} get $name
            print $"Setting ($name) to ($value)"
            load-env {{ $name: $value }}
        }} else {{
            for var in ({sv_path} get-all | lines) {{
                let parts = $var | split row -n 2 "="
                let name = $parts.0
                let value = {sv_path} decode-value $parts.1
                print $"Setting ($name) to ($value)"
                load-env {{ $name: $value }}
            }}
        }}
    }}

    # Open the database of variables and return a table containing each
    # variable's name, value, and modified time (in seconds since Unix epoch)
    export def --env list [] {{
        open ({sv_path} db-path)
    }}
}}
"#)
}
