#nullable enable

using System.Text;
using UnityEditor;
using UnityEditor.TestTools.TestRunner.Api;
using UnityEngine;

public static class RunAllTestsCommand
{
    [MenuItem("Tools/Run All Tests")]
    public static void RunAllTests()
    {
        Debug.Log("[TestRunner] Run started");

        var api = ScriptableObject.CreateInstance<TestRunnerApi>();
        api.RegisterCallbacks(new TestCallbacks());

        var filter = new Filter { testMode = TestMode.EditMode };
        api.Execute(new ExecutionSettings(filter));
    }

    class TestCallbacks : ICallbacks
    {
        int _passed;
        int _failed;
        int _skipped;
        readonly StringBuilder _failures = new();

        public void RunStarted(ITestAdaptor testsToRun)
        {
        }

        public void TestStarted(ITestAdaptor test)
        {
        }

        public void TestFinished(ITestResultAdaptor result)
        {
            if (result.Test.IsSuite) return;

            switch (result.TestStatus)
            {
                case TestStatus.Passed:
                    _passed++;
                    Debug.Log(
                        $"[TestRunner] PASS: {result.Test.FullName} ({result.Duration:F3}s)");
                    break;
                case TestStatus.Failed:
                    _failed++;
                    var message = result.Message?.Replace("\n", " ").Replace("\r", "");
                    Debug.LogError(
                        $"[TestRunner] FAIL: {result.Test.FullName} ({result.Duration:F3}s) - {message}");
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
            var total = _passed + _failed + _skipped;
            if (_failed > 0)
            {
                Debug.LogError(
                    $"[TestRunner] Run finished: {_passed} passed, {_failed} failed, {_skipped} skipped (total: {total})");
                Debug.LogError($"[TestRunner] Failures:\n{_failures}");
            }
            else
            {
                Debug.Log(
                    $"[TestRunner] Run finished: {_passed} passed, {_failed} failed, {_skipped} skipped (total: {total})");
            }
        }
    }
}
