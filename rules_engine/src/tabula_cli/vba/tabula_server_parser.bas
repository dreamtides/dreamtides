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
    Dim bytes() As Byte
    Dim byteCount As Long
    Dim i As Long
    Dim c As String
    Dim hexStr As String
    Dim byteValue As Byte

    ReDim bytes(0 To Len(encoded) * 3)
    byteCount = 0
    i = 1

    Do While i <= Len(encoded)
        c = Mid(encoded, i, 1)
        If c = "%" Then
            If i + 2 <= Len(encoded) Then
                hexStr = Mid(encoded, i + 1, 2)
                byteValue = CByte(CLng("&H" & hexStr))
                bytes(byteCount) = byteValue
                byteCount = byteCount + 1
                i = i + 3
            Else
                bytes(byteCount) = Asc(c)
                byteCount = byteCount + 1
                i = i + 1
            End If
        Else
            bytes(byteCount) = Asc(c)
            byteCount = byteCount + 1
            i = i + 1
        End If
    Loop

    If byteCount > 0 Then
        ReDim Preserve bytes(0 To byteCount - 1)
        PercentDecode = Utf8BytesToString(bytes)
    Else
        PercentDecode = ""
    End If
End Function

Private Function Utf8BytesToString(bytes() As Byte) As String
    Dim result As String
    Dim i As Long
    Dim b1 As Long
    Dim b2 As Long
    Dim b3 As Long
    Dim b4 As Long
    Dim codePoint As Long
    Dim upperBound As Long

    result = ""
    i = LBound(bytes)
    upperBound = UBound(bytes)

    Do While i <= upperBound
        b1 = bytes(i)

        If (b1 And &H80) = 0 Then
            result = result & ChrW(b1)
            i = i + 1
        ElseIf (b1 And &HE0) = &HC0 Then
            If i + 1 <= upperBound Then
                b2 = bytes(i + 1)
                codePoint = ((b1 And &H1F) * 64) + (b2 And &H3F)
                result = result & ChrW(codePoint)
                i = i + 2
            Else
                result = result & "?"
                i = i + 1
            End If
        ElseIf (b1 And &HF0) = &HE0 Then
            If i + 2 <= upperBound Then
                b2 = bytes(i + 1)
                b3 = bytes(i + 2)
                codePoint = ((b1 And &HF) * 4096) + ((b2 And &H3F) * 64) + (b3 And &H3F)
                result = result & ChrW(codePoint)
                i = i + 3
            Else
                result = result & "?"
                i = i + 1
            End If
        ElseIf (b1 And &HF8) = &HF0 Then
            If i + 3 <= upperBound Then
                b2 = bytes(i + 1)
                b3 = bytes(i + 2)
                b4 = bytes(i + 3)
                codePoint = ((b1 And &H7) * 262144) + ((b2 And &H3F) * 4096) + ((b3 And &H3F) * 64) + (b4 And &H3F)
                If codePoint > 65535 Then
                    result = result & "?"
                Else
                    result = result & ChrW(codePoint)
                End If
                i = i + 4
            Else
                result = result & "?"
                i = i + 1
            End If
        Else
            result = result & "?"
            i = i + 1
        End If
    Loop

    Utf8BytesToString = result
End Function

