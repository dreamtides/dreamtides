#nullable enable

using System.Text;
using UnityEditor;
using UnityEditor.TestTools.TestRunner.Api;
using UnityEngine;

public static class RunAllTestsCommand
{
  // Monotonically increasing run token. Each RunAllTests call increments
  // this and stamps the token into the active callback. Stale callbacks
  // left over from previous runs (or from pre-domain-reload
  // ScriptableObjects) will have an outdated token, so their
  // TestFinished/RunFinished invocations are silently ignored.
  static int _runToken;

  [MenuItem("Tools/Run All Tests")]
  public static void RunAllTests()
  {
    // Destroy every TestRunnerApi ScriptableObject that survived a
    // domain reload. After a reload, C# static fields reset to null
    // but ScriptableObjects stay alive with their registered callbacks,
    // causing duplicate counting (310 real tests reported as thousands).
    foreach (var stale in Resources.FindObjectsOfTypeAll<TestRunnerApi>())
    {
      Object.DestroyImmediate(stale);
    }

    var token = ++_runToken;

    // Use Debug.Log (not LogError) for all TestRunner output. Unity's
    // test framework treats any Debug.LogError emitted during a test as
    // an "unhandled log message" failure. When our FAIL line is logged
    // via LogError, the *next* test inherits it and fails with
    // "Unhandled log message", creating a cascade where one real failure
    // causes every subsequent test to fail.
    Debug.Log("[TestRunner] Run started");

    var api = ScriptableObject.CreateInstance<TestRunnerApi>();
    api.RegisterCallbacks(new TestCallbacks(token));

    var filter = new Filter { testMode = TestMode.EditMode };
    api.Execute(new ExecutionSettings(filter));
  }

  class TestCallbacks : ICallbacks
  {
    readonly int _token;
    int _passed;
    int _failed;
    int _skipped;
    readonly StringBuilder _failures = new();

    public TestCallbacks(int token)
    {
      _token = token;
    }

    bool IsStale => _token != _runToken;

    public void RunStarted(ITestAdaptor testsToRun) { }

    public void TestStarted(ITestAdaptor test) { }

    public void TestFinished(ITestResultAdaptor result)
    {
      if (IsStale || result.Test.IsSuite)
        return;

      switch (result.TestStatus)
      {
        case TestStatus.Passed:
          _passed++;
          Debug.Log($"[TestRunner] PASS: {result.Test.FullName} ({result.Duration:F3}s)");
          break;
        case TestStatus.Failed:
          _failed++;
          var message = result.Message?.Replace("\n", " ").Replace("\r", "");
          Debug.Log(
            $"[TestRunner] FAIL: {result.Test.FullName} ({result.Duration:F3}s) - {message}"
          );
          _failures.AppendLine($"  {result.Test.FullName}: {message}");
          break;
        case TestStatus.Skipped:
          _skipped++;
          Debug.Log($"[TestRunner] SKIP: {result.Test.FullName}");
          break;
        case TestStatus.Inconclusive:
          _skipped++;
          Debug.Log($"[TestRunner] INCONCLUSIVE: {result.Test.FullName}");
          break;
      }
    }

    public void RunFinished(ITestResultAdaptor result)
    {
      if (IsStale)
        return;

      var total = _passed + _failed + _skipped;
      Debug.Log(
        $"[TestRunner] Run finished: {_passed} passed, {_failed} failed, {_skipped} skipped (total: {total})"
      );
      if (_failed > 0)
      {
        Debug.Log($"[TestRunner] Failures:\n{_failures}");
      }
    }
  }
}
