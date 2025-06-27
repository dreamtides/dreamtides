#nullable enable

using System;

namespace Dreamtides.Services
{
  public class TestConfiguration
  {
    public string? TestScenario { get; }
    public Guid? IntegrationTestId { get; }

    public TestConfiguration(string testScenario)
    {
      TestScenario = testScenario;
      IntegrationTestId = null;
    }

    public TestConfiguration(Guid integrationTestId)
    {
      TestScenario = null;
      IntegrationTestId = integrationTestId;
    }
  }
}