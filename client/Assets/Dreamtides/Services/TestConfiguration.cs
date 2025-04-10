#nullable enable

namespace Dreamtides.Services
{
  public class TestConfiguration
  {
    public string TestScenario { get; }

    public TestConfiguration(string testScenario)
    {
      TestScenario = testScenario;
    }
  }
}