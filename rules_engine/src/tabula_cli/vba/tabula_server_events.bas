Option Explicit

Private Const CHANGESET_ID_NAME As String = "TabulaLastChangesetId"
Private Const SERVER_ENABLED_NAME As String = "TabulaServerEnabled"

Private Sub Workbook_AfterSave(ByVal Success As Boolean)
    If Not Success Then
        Exit Sub
    End If

    If Not IsServerEnabled() Then
        Exit Sub
    End If

    Dim workbookPath As String
    Dim workbookMtime As Long
    Dim workbookSize As Long
    Dim responseText As String
    Dim response As TabulaResponse

    workbookPath = ThisWorkbook.FullName
    workbookMtime = GetFileMtime(workbookPath)
    workbookSize = FileLen(workbookPath)

    responseText = SendTabulaRequest(workbookPath, workbookMtime, workbookSize, "")

    If responseText = "" Then
        Exit Sub
    End If

    response = ParseTabulaResponse(responseText)

    If response.Status <> "ok" Then
        Exit Sub
    End If

    If response.ChangesetId <> "" Then
        If response.ChangesetId = GetLastChangesetId() Then
            Exit Sub
        End If
        SetLastChangesetId response.ChangesetId
    End If

    ApplyChanges response

    Application.ScreenUpdating = True
    Application.EnableEvents = True
    Application.Calculation = xlCalculationAutomatic
End Sub

Private Function IsServerEnabled() As Boolean
    Dim name As Name
    On Error Resume Next
    Set name = ThisWorkbook.Names(SERVER_ENABLED_NAME)
    If name Is Nothing Then
        IsServerEnabled = True
    Else
        IsServerEnabled = (name.RefersTo = "=TRUE")
    End If
    On Error GoTo 0
End Function

Private Function GetLastChangesetId() As String
    Dim name As Name
    On Error Resume Next
    Set name = ThisWorkbook.Names(CHANGESET_ID_NAME)
    If name Is Nothing Then
        GetLastChangesetId = ""
    Else
        Dim refersTo As String
        refersTo = name.RefersTo
        If Left(refersTo, 1) = "=" Then
            refersTo = Mid(refersTo, 2)
        End If
        If Left(refersTo, 1) = """" And Right(refersTo, 1) = """" Then
            refersTo = Mid(refersTo, 2, Len(refersTo) - 2)
        End If
        GetLastChangesetId = refersTo
    End If
    On Error GoTo 0
End Function

Private Sub SetLastChangesetId(changesetId As String)
    Dim name As Name
    On Error Resume Next
    Set name = ThisWorkbook.Names(CHANGESET_ID_NAME)
    If name Is Nothing Then
        ThisWorkbook.Names.Add Name:=CHANGESET_ID_NAME, RefersTo:="=""" & changesetId & """"
    Else
        name.RefersTo = "=""" & changesetId & """"
    End If
    On Error GoTo 0
End Sub

Public Sub EnableTabulaServer()
    Dim name As Name
    On Error Resume Next
    Set name = ThisWorkbook.Names(SERVER_ENABLED_NAME)
    If name Is Nothing Then
        ThisWorkbook.Names.Add Name:=SERVER_ENABLED_NAME, RefersTo:="=TRUE"
    Else
        name.RefersTo = "=TRUE"
    End If
    On Error GoTo 0
    MsgBox "Tabula server integration enabled.", vbInformation, "Tabula Server"
End Sub

Public Sub DisableTabulaServer()
    Dim name As Name
    On Error Resume Next
    Set name = ThisWorkbook.Names(SERVER_ENABLED_NAME)
    If name Is Nothing Then
        ThisWorkbook.Names.Add Name:=SERVER_ENABLED_NAME, RefersTo:="=FALSE"
    Else
        name.RefersTo = "=FALSE"
    End If
    On Error GoTo 0
    MsgBox "Tabula server integration disabled.", vbInformation, "Tabula Server"
End Sub

Private Function GetFileMtime(filePath As String) As Long
    Dim fileDate As Date
    Dim unixEpoch As Date
    Dim secondsSinceEpoch As Double

    On Error Resume Next
    fileDate = FileDateTime(filePath)
    If Err.Number <> 0 Then
        GetFileMtime = 0
        Exit Function
    End If
    On Error GoTo 0

    unixEpoch = DateSerial(1970, 1, 1)
    secondsSinceEpoch = (fileDate - unixEpoch) * 86400#

    GetFileMtime = CLng(secondsSinceEpoch)
End Function

