#nullable enable

using System;

namespace Dreamtides.Services
{
  public class TestConfiguration
  {
    public Guid IntegrationTestId { get; }

    public TestConfiguration(Guid integrationTestId)
    {
      IntegrationTestId = integrationTestId;
    }
  }
}