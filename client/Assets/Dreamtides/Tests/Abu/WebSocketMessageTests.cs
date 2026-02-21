#nullable enable

using System.Collections.Generic;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using NUnit.Framework;

namespace Abu.Tests
{
    public class WebSocketMessageTests
    {
        [Test]
        public void SerializeSuccessResponse()
        {
            var response = new AbuResponse
            {
                Id = "resp-1",
                Success = true,
                Data = new { clicked = true },
            };

            var json = JsonConvert.SerializeObject(response);
            var parsed = JObject.Parse(json);

            Assert.AreEqual("resp-1", parsed["id"]!.Value<string>());
            Assert.AreEqual(true, parsed["success"]!.Value<bool>());
            Assert.IsNotNull(parsed["data"]);
            Assert.AreEqual(true, parsed["data"]!["clicked"]!.Value<bool>());
            Assert.IsFalse(parsed.ContainsKey("error"));
        }

        [Test]
        public void SerializeErrorResponse()
        {
            var response = new AbuResponse
            {
                Id = "resp-2",
                Success = false,
                Error = "Ref e3 not found",
            };

            var json = JsonConvert.SerializeObject(response);
            var parsed = JObject.Parse(json);

            Assert.AreEqual("resp-2", parsed["id"]!.Value<string>());
            Assert.AreEqual(false, parsed["success"]!.Value<bool>());
            Assert.AreEqual("Ref e3 not found", parsed["error"]!.Value<string>());
            Assert.IsFalse(parsed.ContainsKey("data"));
        }

        [Test]
        public void RoundTripResponse()
        {
            var original = new AbuResponse
            {
                Id = "round-1",
                Success = true,
                Data = new { hovered = true },
            };

            var json = JsonConvert.SerializeObject(original);
            var deserialized = JsonConvert.DeserializeObject<AbuResponse>(json);

            Assert.IsNotNull(deserialized);
            Assert.AreEqual("round-1", deserialized!.Id);
            Assert.AreEqual(true, deserialized.Success);
            Assert.IsNotNull(deserialized.Data);
            Assert.AreEqual(true, ((JObject)deserialized.Data!)["hovered"]!.Value<bool>());
            Assert.IsNull(deserialized.Error);
        }

        [Test]
        public void RoundTripErrorResponse()
        {
            var original = new AbuResponse
            {
                Id = "round-err",
                Success = false,
                Error = "Something went wrong",
            };

            var json = JsonConvert.SerializeObject(original);
            var deserialized = JsonConvert.DeserializeObject<AbuResponse>(json);

            Assert.IsNotNull(deserialized);
            Assert.AreEqual("round-err", deserialized!.Id);
            Assert.AreEqual(false, deserialized.Success);
            Assert.IsNull(deserialized.Data);
            Assert.AreEqual("Something went wrong", deserialized.Error);
        }

        [Test]
        public void SnapshotDataSerialization()
        {
            var snapshotData = new SnapshotData
            {
                Nodes = new List<AbuSceneNode>
                {
                    new AbuSceneNode
                    {
                        Role = "application",
                        Label = "TestApp",
                        Interactive = false,
                        Children = new List<AbuSceneNode>
                        {
                            new AbuSceneNode
                            {
                                Role = "button",
                                Label = "OK",
                                Interactive = true,
                                Children = new List<AbuSceneNode>(),
                            },
                        },
                    },
                },
            };

            var response = new AbuResponse
            {
                Id = "snap-1",
                Success = true,
                Data = snapshotData,
            };

            var json = JsonConvert.SerializeObject(response);
            var parsed = JObject.Parse(json);

            Assert.AreEqual(true, parsed["success"]!.Value<bool>());
            var data = parsed["data"];
            Assert.IsNotNull(data);
            var nodes = data!["nodes"] as JArray;
            Assert.IsNotNull(nodes);
            Assert.AreEqual(1, nodes!.Count);
            Assert.AreEqual("application", nodes[0]["role"]!.Value<string>());
            Assert.AreEqual("TestApp", nodes[0]["label"]!.Value<string>());

            var childNodes = nodes[0]["children"] as JArray;
            Assert.IsNotNull(childNodes);
            Assert.AreEqual(1, childNodes!.Count);
            Assert.AreEqual("button", childNodes[0]["role"]!.Value<string>());
            Assert.AreEqual("OK", childNodes[0]["label"]!.Value<string>());
            Assert.AreEqual(true, childNodes[0]["interactive"]!.Value<bool>());
        }

        [Test]
        public void ResponseCamelCaseFieldNames()
        {
            var response = new AbuResponse
            {
                Id = "case-1",
                Success = true,
                Data = new { someValue = 42 },
            };

            var json = JsonConvert.SerializeObject(response);

            Assert.IsTrue(json.Contains("\"id\""));
            Assert.IsTrue(json.Contains("\"success\""));
            Assert.IsTrue(json.Contains("\"data\""));
            Assert.IsFalse(json.Contains("\"Id\""));
            Assert.IsFalse(json.Contains("\"Success\""));
            Assert.IsFalse(json.Contains("\"Data\""));
        }

        [Test]
        public void DefaultCommandHandlerReturnsError()
        {
            var handler = new DefaultCommandHandler();
            var command = new AbuCommand { Id = "test-1", Command = "click" };

            AbuResponse? result = null;
            handler.HandleCommand(
                command,
                null!,
                response =>
                {
                    result = response;
                }
            );

            Assert.IsNotNull(result);
            Assert.AreEqual("test-1", result!.Id);
            Assert.AreEqual(false, result.Success);
            Assert.IsNotNull(result.Error);
            Assert.IsTrue(result.Error!.Contains("click"));
        }
    }
}
