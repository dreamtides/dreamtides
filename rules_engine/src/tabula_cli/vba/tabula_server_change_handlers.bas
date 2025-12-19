Option Explicit

Public Sub ApplyChanges(response As TabulaResponse)
    Dim eventsEnabled As Boolean
    Dim screenUpdating As Boolean
    Dim calculationMode As XlCalculation

    eventsEnabled = Application.EnableEvents
    screenUpdating = Application.ScreenUpdating
    calculationMode = Application.Calculation

    Application.EnableEvents = False
    Application.ScreenUpdating = False
    Application.Calculation = xlCalculationManual

    On Error GoTo ErrorHandler

    Dim i As Long
    For i = LBound(response.Changes) To UBound(response.Changes)
        Dim change As TabulaChange
        change = response.Changes(i)

        Select Case change.ChangeType
            Case "set_bold"
                ApplySetBold change
            Case "set_font_color_spans"
                ApplySetFontColorSpans change
            Case "set_value"
                ApplySetValue change
            Case "clear_value"
                ApplyClearValue change
            Case "set_font_color"
                ApplySetFontColor change
            Case "set_font_size"
                ApplySetFontSize change
            Case "set_fill_color"
                ApplySetFillColor change
            Case "set_number_format"
                ApplySetNumberFormat change
            Case "set_horizontal_alignment"
                ApplySetHorizontalAlignment change
            Case "set_italic"
                ApplySetItalic change
            Case "set_underline"
                ApplySetUnderline change
            Case "set_font_name_spans"
                ApplySetFontNameSpans change
            Case "set_font_size_spans"
                ApplySetFontSizeSpans change
        End Select
    Next i

    GoTo Cleanup

ErrorHandler:
    MsgBox "Error applying Tabula server changes: " & Err.Description, vbCritical, "Tabula Server Error"

Cleanup:
    Application.EnableEvents = eventsEnabled
    Application.ScreenUpdating = screenUpdating
    Application.Calculation = calculationMode
End Sub

Private Sub ApplySetBold(change As TabulaChange)
    Dim sheet As Worksheet
    Dim cell As Range

    On Error Resume Next
    Set sheet = ThisWorkbook.Worksheets(change.Sheet)
    If sheet Is Nothing Then
        Exit Sub
    End If

    Set cell = sheet.Range(change.Cell)
    If cell Is Nothing Then
        Exit Sub
    End If

    On Error GoTo 0

    If change.Value1 = "1" Then
        cell.Font.Bold = True
    Else
        cell.Font.Bold = False
    End If
End Sub

Private Sub ApplySetFontColorSpans(change As TabulaChange)
    Dim sheet As Worksheet
    Dim cell As Range
    Dim spans() As String
    Dim i As Long
    Dim spanParts() As String
    Dim startPos As Long
    Dim length As Long
    Dim rgbValue As Long

    On Error Resume Next
    Set sheet = ThisWorkbook.Worksheets(change.Sheet)
    If sheet Is Nothing Then
        Exit Sub
    End If

    Set cell = sheet.Range(change.Cell)
    If cell Is Nothing Then
        Exit Sub
    End If

    On Error GoTo 0

    rgbValue = HexToLong(change.Value1)
    spans = Split(change.Value2, ",")

    For i = LBound(spans) To UBound(spans)
        spanParts = Split(spans(i), ":")
        If UBound(spanParts) >= 1 Then
            startPos = CLng(spanParts(0))
            length = CLng(spanParts(1))
            cell.Characters(startPos, length).Font.Color = rgbValue
        End If
    Next i
End Sub

Private Sub ApplySetValue(change As TabulaChange)
    Dim sheet As Worksheet
    Dim cell As Range

    On Error Resume Next
    Set sheet = ThisWorkbook.Worksheets(change.Sheet)
    If sheet Is Nothing Then
        Exit Sub
    End If

    Set cell = sheet.Range(change.Cell)
    If cell Is Nothing Then
        Exit Sub
    End If

    On Error GoTo 0

    cell.Value = change.Value1
End Sub

Private Sub ApplyClearValue(change As TabulaChange)
    Dim sheet As Worksheet
    Dim cell As Range

    On Error Resume Next
    Set sheet = ThisWorkbook.Worksheets(change.Sheet)
    If sheet Is Nothing Then
        Exit Sub
    End If

    Set cell = sheet.Range(change.Cell)
    If cell Is Nothing Then
        Exit Sub
    End If

    On Error GoTo 0

    cell.ClearContents
End Sub

Private Sub ApplySetFontColor(change As TabulaChange)
    Dim sheet As Worksheet
    Dim cell As Range
    Dim rgbValue As Long

    On Error Resume Next
    Set sheet = ThisWorkbook.Worksheets(change.Sheet)
    If sheet Is Nothing Then
        Exit Sub
    End If

    Set cell = sheet.Range(change.Cell)
    If cell Is Nothing Then
        Exit Sub
    End If

    On Error GoTo 0

    rgbValue = HexToLong(change.Value1)
    cell.Font.Color = rgbValue
End Sub

Private Sub ApplySetFontSize(change As TabulaChange)
    Dim sheet As Worksheet
    Dim cell As Range
    Dim points As Double

    On Error Resume Next
    Set sheet = ThisWorkbook.Worksheets(change.Sheet)
    If sheet Is Nothing Then
        Exit Sub
    End If

    Set cell = sheet.Range(change.Cell)
    If cell Is Nothing Then
        Exit Sub
    End If

    On Error GoTo 0

    points = CDbl(change.Value1)
    cell.Font.Size = points
End Sub

Private Sub ApplySetFillColor(change As TabulaChange)
    Dim sheet As Worksheet
    Dim cell As Range
    Dim rgbValue As Long

    On Error Resume Next
    Set sheet = ThisWorkbook.Worksheets(change.Sheet)
    If sheet Is Nothing Then
        Exit Sub
    End If

    Set cell = sheet.Range(change.Cell)
    If cell Is Nothing Then
        Exit Sub
    End If

    On Error GoTo 0

    rgbValue = HexToLong(change.Value1)
    cell.Interior.Color = rgbValue
End Sub

Private Sub ApplySetNumberFormat(change As TabulaChange)
    Dim sheet As Worksheet
    Dim cell As Range

    On Error Resume Next
    Set sheet = ThisWorkbook.Worksheets(change.Sheet)
    If sheet Is Nothing Then
        Exit Sub
    End If

    Set cell = sheet.Range(change.Cell)
    If cell Is Nothing Then
        Exit Sub
    End If

    On Error GoTo 0

    cell.NumberFormat = change.Value1
End Sub

Private Sub ApplySetHorizontalAlignment(change As TabulaChange)
    Dim sheet As Worksheet
    Dim cell As Range
    Dim alignment As XlHAlign

    On Error Resume Next
    Set sheet = ThisWorkbook.Worksheets(change.Sheet)
    If sheet Is Nothing Then
        Exit Sub
    End If

    Set cell = sheet.Range(change.Cell)
    If cell Is Nothing Then
        Exit Sub
    End If

    On Error GoTo 0

    Select Case LCase(change.Value1)
        Case "left"
            alignment = xlHAlignLeft
        Case "center"
            alignment = xlHAlignCenter
        Case "right"
            alignment = xlHAlignRight
        Case Else
            Exit Sub
    End Select

    cell.HorizontalAlignment = alignment
End Sub

Private Sub ApplySetItalic(change As TabulaChange)
    Dim sheet As Worksheet
    Dim cell As Range

    On Error Resume Next
    Set sheet = ThisWorkbook.Worksheets(change.Sheet)
    If sheet Is Nothing Then
        Exit Sub
    End If

    Set cell = sheet.Range(change.Cell)
    If cell Is Nothing Then
        Exit Sub
    End If

    On Error GoTo 0

    If change.Value1 = "1" Then
        cell.Font.Italic = True
    Else
        cell.Font.Italic = False
    End If
End Sub

Private Sub ApplySetUnderline(change As TabulaChange)
    Dim sheet As Worksheet
    Dim cell As Range

    On Error Resume Next
    Set sheet = ThisWorkbook.Worksheets(change.Sheet)
    If sheet Is Nothing Then
        Exit Sub
    End If

    Set cell = sheet.Range(change.Cell)
    If cell Is Nothing Then
        Exit Sub
    End If

    On Error GoTo 0

    If change.Value1 = "1" Then
        cell.Font.Underline = xlUnderlineStyleSingle
    Else
        cell.Font.Underline = xlUnderlineStyleNone
    End If
End Sub

Private Sub ApplySetFontNameSpans(change As TabulaChange)
    Dim sheet As Worksheet
    Dim cell As Range
    Dim spans() As String
    Dim i As Long
    Dim spanParts() As String
    Dim startPos As Long
    Dim length As Long
    Dim fontName As String

    On Error Resume Next
    Set sheet = ThisWorkbook.Worksheets(change.Sheet)
    If sheet Is Nothing Then
        Exit Sub
    End If

    Set cell = sheet.Range(change.Cell)
    If cell Is Nothing Then
        Exit Sub
    End If

    On Error GoTo 0

    cell.ReadingOrder = xlLTR

    fontName = change.Value1
    spans = Split(change.Value2, ",")

    For i = LBound(spans) To UBound(spans)
        spanParts = Split(spans(i), ":")
        If UBound(spanParts) >= 1 Then
            startPos = CLng(spanParts(0))
            length = CLng(spanParts(1))
            cell.Characters(startPos, length).Font.Name = fontName
        End If
    Next i
End Sub

Private Sub ApplySetFontSizeSpans(change As TabulaChange)
    Dim sheet As Worksheet
    Dim cell As Range
    Dim spans() As String
    Dim i As Long
    Dim spanParts() As String
    Dim startPos As Long
    Dim length As Long
    Dim points As Double

    On Error Resume Next
    Set sheet = ThisWorkbook.Worksheets(change.Sheet)
    If sheet Is Nothing Then
        Exit Sub
    End If

    Set cell = sheet.Range(change.Cell)
    If cell Is Nothing Then
        Exit Sub
    End If

    On Error GoTo 0

    points = CDbl(change.Value1)
    spans = Split(change.Value2, ",")

    For i = LBound(spans) To UBound(spans)
        spanParts = Split(spans(i), ":")
        If UBound(spanParts) >= 1 Then
            startPos = CLng(spanParts(0))
            length = CLng(spanParts(1))
            cell.Characters(startPos, length).Font.Size = points
        End If
    Next i
End Sub

Private Function HexToLong(hexStr As String) As Long
    Dim r As Long
    Dim g As Long
    Dim b As Long

    If Len(hexStr) <> 6 Then
        HexToLong = 0
        Exit Function
    End If

    r = CLng("&H" & Mid(hexStr, 1, 2))
    g = CLng("&H" & Mid(hexStr, 3, 2))
    b = CLng("&H" & Mid(hexStr, 5, 2))

    HexToLong = RGB(r, g, b)
End Function

