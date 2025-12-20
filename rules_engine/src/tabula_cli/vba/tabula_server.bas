Option Explicit

Private Const SERVER_URL As String = "http://127.0.0.1:3030/notify"
Private Const MAX_RETRIES As Long = 3
Private Const DEFAULT_RETRY_DELAY_MS As Long = 500

Public Function SendTabulaRequest(workbookPath As String, workbookMtime As Long, workbookSize As Long, Optional changedRange As String = "") As String
    Dim requestText As String
    Dim requestId As String
    Dim tempDir As String
    Dim requestPath As String
    Dim responsePath As String
    Dim responseText As String
    Dim retryCount As Long
    Dim retryAfterMs As Long
    Dim success As Boolean

    requestId = GenerateRequestId()
    requestText = BuildRequest(requestId, workbookPath, workbookMtime, workbookSize, changedRange)

    tempDir = Environ("TMPDIR")
    If tempDir = "" Then
        tempDir = Environ("TMP")
    End If
    If tempDir = "" Then
        tempDir = "/tmp"
    End If

    requestPath = tempDir & "/tabula_request_" & requestId & ".txt"
    responsePath = tempDir & "/tabula_response_" & requestId & ".txt"

    retryCount = 0
    success = False

    Do While retryCount <= MAX_RETRIES And Not success
        On Error Resume Next
        Kill responsePath
        On Error GoTo 0

        WriteTextFile requestPath, requestText

        Dim result As String
        result = AppleScriptTask("tabula_server_curl.scpt", "tabula_server_request", SERVER_URL & "|" & requestPath & "|" & responsePath)

        If Left(result, 2) = "ok" Then
            Dim httpCode As String
            httpCode = Mid(result, 4)
            If httpCode = "200" Then
                responseText = ReadTextFile(responsePath)
                If responseText <> "" Then
                    success = True
                Else
                    retryAfterMs = DEFAULT_RETRY_DELAY_MS
                End If
            Else
                retryAfterMs = DEFAULT_RETRY_DELAY_MS
            End If
        Else
            retryAfterMs = DEFAULT_RETRY_DELAY_MS
        End If

        If Not success Then
            Dim parsedResponse As TabulaResponse
            If responseText <> "" Then
                parsedResponse = ParseTabulaResponse(responseText)
                If parsedResponse.RetryAfterMs > 0 Then
                    retryAfterMs = parsedResponse.RetryAfterMs
                End If
            End If

            retryCount = retryCount + 1
            If retryCount <= MAX_RETRIES Then
                Application.Wait (Now + TimeValue("0:00:00") * (retryAfterMs / 1000#))
            End If
        End If
    Loop

    On Error Resume Next
    Kill requestPath
    Kill responsePath
    On Error GoTo 0

    If success Then
        SendTabulaRequest = responseText
    Else
        SendTabulaRequest = ""
    End If
End Function

Private Function BuildRequest(requestId As String, workbookPath As String, workbookMtime As Long, workbookSize As Long, changedRange As String) As String
    Dim lines() As String
    Dim lineCount As Long

    ReDim lines(0)
    lines(0) = "TABULA/1"
    lineCount = 1

    ReDim Preserve lines(lineCount)
    lines(lineCount) = "REQUEST_ID " & PercentEncode(requestId)
    lineCount = lineCount + 1

    ReDim Preserve lines(lineCount)
    lines(lineCount) = "WORKBOOK_PATH " & PercentEncode(workbookPath)
    lineCount = lineCount + 1

    ReDim Preserve lines(lineCount)
    lines(lineCount) = "WORKBOOK_MTIME " & CStr(workbookMtime)
    lineCount = lineCount + 1

    ReDim Preserve lines(lineCount)
    lines(lineCount) = "WORKBOOK_SIZE " & CStr(workbookSize)
    lineCount = lineCount + 1

    If changedRange <> "" Then
        Dim rangeParts() As String
        rangeParts = Split(changedRange, "!", 2)
        If UBound(rangeParts) >= 1 Then
            ReDim Preserve lines(lineCount)
            lines(lineCount) = "CHANGED_RANGE " & PercentEncode(rangeParts(0)) & " " & PercentEncode(rangeParts(1))
            lineCount = lineCount + 1
        End If
    End If

    BuildRequest = Join(lines, vbLf) & vbLf
End Function

Private Function GenerateRequestId() As String
    Dim timestamp As String
    Dim randomPart As String
    Dim i As Long

    timestamp = Format(Now, "yyyymmddhhnnss") & Format(Timer, "000")

    randomPart = ""
    Randomize
    For i = 1 To 8
        randomPart = randomPart & Hex(Int(Rnd() * 16))
    Next i

    GenerateRequestId = timestamp & randomPart
End Function

Private Function PercentEncode(text As String) As String
    Dim result As String
    Dim i As Long
    Dim c As String
    Dim asciiVal As Long

    result = ""
    For i = 1 To Len(text)
        c = Mid(text, i, 1)
        asciiVal = Asc(c)
        If c = " " Then
            result = result & "%20"
        ElseIf c = "%" Then
            result = result & "%25"
        ElseIf c = vbLf Then
            result = result & "%0A"
        ElseIf c = vbCr Then
            result = result & "%0D"
        ElseIf asciiVal >= 32 And asciiVal <= 126 Then
            result = result & c
        Else
            result = result & "%" & Right("0" & Hex(asciiVal), 2)
        End If
    Next i

    PercentEncode = result
End Function

Private Sub WriteTextFile(filePath As String, content As String)
    Dim fileNum As Integer
    fileNum = FreeFile
    Open filePath For Output As #fileNum
    Print #fileNum, content;
    Close #fileNum
End Sub

Private Function ReadTextFile(filePath As String) As String
    Dim fileNum As Integer
    Dim content As String
    Dim line As String

    On Error GoTo ErrorHandler

    fileNum = FreeFile
    Open filePath For Input As #fileNum
    content = ""
    Do While Not EOF(fileNum)
        Line Input #fileNum, line
        If content <> "" Then
            content = content & vbLf
        End If
        content = content & line
    Loop
    Close #fileNum

    ReadTextFile = content
    Exit Function

ErrorHandler:
    On Error Resume Next
    Close #fileNum
    ReadTextFile = ""
End Function

