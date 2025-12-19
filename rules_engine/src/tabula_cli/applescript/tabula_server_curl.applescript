on tabula_server_request(parameters)
    set old_delimiters to AppleScript's text item delimiters
    set AppleScript's text item delimiters to "|"
    set parts to text items of parameters
    set AppleScript's text item delimiters to old_delimiters
    if (count of parts) < 3 then
        return "error|expected url|request_path|response_path"
    end if
    set request_url to item 1 of parts
    set request_path to item 2 of parts
    set response_path to item 3 of parts
    set curl_path to "/usr/bin/curl"
    set curl_command to curl_path & " --silent --show-error --fail --max-time 10 --connect-timeout 5 -X POST --data-binary @" & quoted form of request_path & " -o " & quoted form of response_path & " -w %{http_code} " & quoted form of request_url
    try
        set http_code to do shell script curl_command
        return "ok|" & http_code
    on error err_message number err_number
        return "error|" & err_number & "|" & err_message
    end try
end tabula_server_request
