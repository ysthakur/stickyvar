//! Setup code for shells

/// Setup code for POSIX-compliant shells
pub fn init_posix(sv_path: &str) -> String {
    format!(r#"sv() {{
    if [ "$1" = "set" ]
    then
        if [ "$#" != 3 ]
        then
            echo "Expected 2 arguments: variable name and value"
        fi
        export $2="$3"
        {sv_path} set "$2" "$3"
    elif [ "$1" = "load" ]
    then
        if [ "$#" = 1 ]
        then
            # Load all variables
            while IFS= read -r line
            do
                # Hopefully this doesn't overwrite existing variables called _stickyvar_name/value
                # Worth considering local - it's not part of POSIX but most shells support it
                _stickyvar_name="$(echo $line | cut -d '=' -f 1)"
                _stickyvar_value="$(echo $line | cut -d '=' -f 2-)"
                _stickyvar_value="$({sv_path} decode-value "$_stickyvar_value")"
                export $_stickyvar_name="$_stickyvar_value"
                echo "Set $_stickyvar_name to $_stickyvar_value"
            done << EOF
`{sv_path} get-all`
EOF
            # We have to use redirection rather than a pipe because
            # a pipe would make the while loop run in a subprocess, meaning
            # environment variables wouldn't be affected in the current process
        elif [ "$#" = 2 ]
        then
            # Load only the given variable
            _stickyvar_value="$({sv_path} get "$2")"
            export $2="$_stickyvar_value"
            echo "Set $2 to $_stickyvar_value"
        else
            echo "Expected either no arguments (to load all variables) or 1 argument (name of single variable to load)"
        fi
    elif [ "$1" = "list" ]
    then
        if [ "$#" != 1 ]
        then
            echo "Expected no arguments"
        fi
        {sv_path} list
    elif [ "$1" = "del" ]
    then
        if [ "$#" != 2 ]
        then
            echo "Expected 1 argument (name of variable to delete)"
        fi
        {sv_path} del "$2" && unset "$2"
    else
        echo "Subcommands: set NAME VALUE, load [NAME], list, del [NAME]"
    fi
}}
"#)
}

/// Setup code for Nushell
pub fn init_nushell(sv_path: &str) -> String {
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

    export def --env del [ name: string ] {{
        {sv_path} del $name
        hide-env $name
    }}

    # Open the database of variables and return a table containing each
    # variable's name, value, and modified time (in seconds since Unix epoch)
    export def --env list [] {{
        open ({sv_path} db-path)
    }}
}}
"#)
}
