Attribute VB_Name = "TabulaServerParser"

Option Explicit

Public Type TabulaChange
    ChangeType As String
    Sheet As String
    Cell As String
    Value1 As String
    Value2 As String
    Value3 As String
End Type

Public Type TabulaResponse
    RequestId As String
    Status As String
    RetryAfterMs As Long
    Warnings() As String
    Changes() As TabulaChange
    ChangesetId As String
End Type

Public Function ParseTabulaResponse(responseText As String) As TabulaResponse
    Dim response As TabulaResponse
    Dim lines() As String
    Dim i As Long
    Dim line As String
    Dim parts() As String
    Dim changeCount As Long
    Dim warningCount As Long

    changeCount = 0
    warningCount = 0
    ReDim response.Warnings(0)
    ReDim response.Changes(0)

    lines = Split(responseText, vbLf)

    For i = LBound(lines) To UBound(lines)
        line = Trim(lines(i))
        If line = "" Then
            GoTo NextLine
        End If

        If line = "TABULA/1" Then
            GoTo NextLine
        End If

        parts = Split(line, " ", 2)
        If UBound(parts) < 1 Then
            GoTo NextLine
        End If

        Select Case parts(0)
            Case "REQUEST_ID"
                response.RequestId = PercentDecode(parts(1))
            Case "STATUS"
                response.Status = parts(1)
            Case "RETRY_AFTER_MS"
                response.RetryAfterMs = CLng(parts(1))
            Case "WARNING"
                ReDim Preserve response.Warnings(warningCount)
                response.Warnings(warningCount) = PercentDecode(parts(1))
                warningCount = warningCount + 1
            Case "CHANGE"
                Dim changeParts() As String
                changeParts = Split(parts(1), " ")
                If UBound(changeParts) >= 2 Then
                    ReDim Preserve response.Changes(changeCount)
                    response.Changes(changeCount).ChangeType = changeParts(0)
                    response.Changes(changeCount).Sheet = PercentDecode(changeParts(1))
                    response.Changes(changeCount).Cell = PercentDecode(changeParts(2))
                    If UBound(changeParts) >= 3 Then
                        response.Changes(changeCount).Value1 = PercentDecode(changeParts(3))
                    End If
                    If UBound(changeParts) >= 4 Then
                        response.Changes(changeCount).Value2 = PercentDecode(changeParts(4))
                    End If
                    If UBound(changeParts) >= 5 Then
                        response.Changes(changeCount).Value3 = PercentDecode(changeParts(5))
                    End If
                    changeCount = changeCount + 1
                End If
            Case "CHANGESET_ID"
                response.ChangesetId = PercentDecode(parts(1))
        End Select

NextLine:
    Next i

    ParseTabulaResponse = response
End Function

Private Function PercentDecode(encoded As String) As String
    Dim result As String
    Dim i As Long
    Dim c As String
    Dim hexStr As String
    Dim decodedChar As String

    result = ""
    i = 1

    Do While i <= Len(encoded)
        c = Mid(encoded, i, 1)
        If c = "%" Then
            If i + 2 <= Len(encoded) Then
                hexStr = Mid(encoded, i + 1, 2)
                decodedChar = Chr(CLng("&H" & hexStr))
                result = result & decodedChar
                i = i + 3
            Else
                result = result & c
                i = i + 1
            End If
        Else
            result = result & c
            i = i + 1
        End If
    Loop

    PercentDecode = result
End Function

