#nullable enable

using System;

namespace Dreamtides.Services
{
  public class TestConfiguration
  {
    public Guid TestId { get; }

    public TestConfiguration(Guid testId)
    {
      TestId = testId;
    }
  }
}
