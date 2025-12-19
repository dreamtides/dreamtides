Option Explicit

Public Sub EnsureTableIDs(ByVal Sh As Worksheet, ByVal Target As Range)
    On Error GoTo Fail
    If Target Is Nothing Then Exit Sub

    Dim lo As ListObject
    Dim hit As Range

    For Each lo In Sh.ListObjects
        If lo.DataBodyRange Is Nothing Then GoTo NextTable

        Set hit = Intersect(Target, lo.DataBodyRange)
        If Not hit Is Nothing Then
            ProcessListObjectRows lo, hit
        End If

NextTable:
        Set hit = Nothing
    Next lo

    Exit Sub

Fail:
    ' Fail closed (do nothing) - event wrapper will re-enable events
End Sub


Private Sub ProcessListObjectRows(ByVal lo As ListObject, ByVal hit As Range)
    On Error GoTo Fail

    Dim idColIndex As Long
    idColIndex = FindTableColumnIndex(lo, "ID")
    If idColIndex = 0 Then Exit Sub

    Dim rowCount As Long
    rowCount = lo.ListRows.Count
    If rowCount <= 0 Then Exit Sub

    Dim touched() As Boolean
    ReDim touched(1 To rowCount) As Boolean

    Dim area As Range, c As Range
    Dim r As Long

    For Each area In hit.Areas
        For Each c In area.Cells
            r = c.Row - lo.DataBodyRange.Row + 1  ' 1-based within DataBodyRange
            If r >= 1 And r <= rowCount Then
                touched(r) = True
            End If
        Next c
    Next area

    Dim rowRange As Range, idCell As Range
    Dim idVal As String, hasOtherContent As Boolean

    For r = 1 To rowCount
        If touched(r) Then
            Set rowRange = lo.ListRows(r).Range
            Set idCell = rowRange.Cells(1, idColIndex)

            idVal = NormalizeBlank(idCell.Value2)
            hasOtherContent = TableRowHasOtherContent(rowRange, idColIndex)

            If (Len(idVal) = 0) And hasOtherContent Then
                idCell.Value2 = RandomGuidV4()
            ElseIf (Len(idVal) > 0) And (Not hasOtherContent) Then
                idCell.ClearContents
            End If
        End If
    Next r

    Exit Sub

Fail:
    ' Do nothing
End Sub


Private Function FindTableColumnIndex(ByVal lo As ListObject, ByVal colName As String) As Long
    Dim lc As ListColumn
    For Each lc In lo.ListColumns
        If StrComp(Trim$(Replace(lc.Name, ChrW(160), " ")), colName, vbTextCompare) = 0 Then
            FindTableColumnIndex = lc.Index
            Exit Function
        End If
    Next lc
    FindTableColumnIndex = 0
End Function


Private Function TableRowHasOtherContent(ByVal rowRange As Range, ByVal idColIndex As Long) As Boolean
    Dim i As Long
    Dim v As Variant
    Dim s As String

    For i = 1 To rowRange.Columns.Count
        If i <> idColIndex Then
            v = rowRange.Cells(1, i).Value2

            If IsError(v) Then
                TableRowHasOtherContent = True
                Exit Function
            End If

            s = NormalizeBlank(v)
            If Len(s) > 0 Then
                TableRowHasOtherContent = True
                Exit Function
            End If
        End If
    Next i

    TableRowHasOtherContent = False
End Function


Private Function NormalizeBlank(ByVal v As Variant) As String
    If IsError(v) Then
        NormalizeBlank = "#ERROR"
        Exit Function
    End If

    Dim s As String
    s = CStr(v)
    s = Replace(s, ChrW(160), " ") ' NBSP -> space
    s = Trim$(s)
    NormalizeBlank = s
End Function


Public Function RandomGuidV4() As String
    Dim b(0 To 15) As Integer
    Dim i As Long

    Randomize
    For i = 0 To 15
        b(i) = Int(Rnd() * 256)
    Next i

    ' Version (4) and Variant (RFC 4122)
    b(6) = (b(6) And &HF) Or &H40
    b(8) = (b(8) And &H3F) Or &H80

    RandomGuidV4 = _
        Hex2(b(0)) & Hex2(b(1)) & Hex2(b(2)) & Hex2(b(3)) & "-" & _
        Hex2(b(4)) & Hex2(b(5)) & "-" & _
        Hex2(b(6)) & Hex2(b(7)) & "-" & _
        Hex2(b(8)) & Hex2(b(9)) & "-" & _
        Hex2(b(10)) & Hex2(b(11)) & Hex2(b(12)) & Hex2(b(13)) & Hex2(b(14)) & Hex2(b(15))
End Function

Private Function Hex2(ByVal n As Integer) As String
    Dim s As String
    s = Hex$(n And &HFF)
    If Len(s) = 1 Then s = "0" & s
    Hex2 = LCase$(s)
End Function
