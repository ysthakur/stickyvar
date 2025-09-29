//! Setup code for POSIX-y shells

pub fn init(sv_path: &str) -> String {
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
            while IFS= read -r line
                do
                    # Hopefully this doesn't overwrite existing variables called _stickyvar_name/value
                    # Worth considering local - it's not part of POSIX but most shells support it
                    _stickyvar_name="$(echo $line | cut -d '=' -f 1)"
                    _stickyvar_value="$(echo $line | cut -d '=' -f 2-)"
                    _stickyvar_value="$({sv_path} decode-value "$_stickyvar_value")"
                    export $_stickyvar_name="$_stickyvar_value"
                    echo "Set $_stickyvar_name to $_stickyvar_value"
                done < <({sv_path} get-all)
                # We have to use process substitution rather than a pipe because
                # a pipe would make the while loop run in a subprocess, meaning
                # environment variables wouldn't be affected in the current process
        elif [ "$#" = 2 ]
        then
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
    else
        echo "Subcommands: set NAME VALUE, load [NAME], list"
    fi
}}
"#)
}
