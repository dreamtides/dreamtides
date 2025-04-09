#nullable enable

using UnityEditor;
using UnityEditor.TestTools.TestRunner.Api;
using UnityEngine;


namespace Dreamtides.Editors
{
  public static class RunTestsMenu
  {
    [MenuItem("Tools/Run All Tests")]
    public static void RunAllTests()
    {
      var testRunnerApi = ScriptableObject.CreateInstance<TestRunnerApi>();
      testRunnerApi.RegisterCallbacks(new TestCallbacks());
      testRunnerApi.Execute(new ExecutionSettings(new Filter()
      {
        testMode = TestMode.PlayMode
      }));
    }

    private class TestCallbacks : ICallbacks
    {
      public void RunStarted(ITestAdaptor testsToRun)
      {
        Debug.Log($"Running all tests at {Screen.width}x{Screen.height}");
      }

      public void RunFinished(ITestResultAdaptor result)
      {
        Debug.Log($"Done running tests. Overall result {result.TestStatus}");
      }

      public void TestStarted(ITestAdaptor test)
      {
      }

      public void TestFinished(ITestResultAdaptor result)
      {
        if (result.TestStatus == TestStatus.Failed)
        {
          Debug.Log($"Failed {result.Name}: {result.Message}");
        }
      }
    }
  }
}