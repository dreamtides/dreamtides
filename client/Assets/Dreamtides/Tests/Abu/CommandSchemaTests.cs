#nullable enable

using System.Collections.Generic;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using NUnit.Framework;

namespace Abu.Tests
{
    public class CommandSchemaTests
    {
        [Test]
        public void DeserializeClickCommand()
        {
            var json = @"{""id"":""abc"",""command"":""click"",""params"":{""ref"":""e3""}}";
            var command = JsonConvert.DeserializeObject<AbuCommand>(json);

            Assert.IsNotNull(command);
            Assert.AreEqual("abc", command!.Id);
            Assert.AreEqual("click", command.Command);
            Assert.IsNotNull(command.Params);
            Assert.AreEqual("e3", command.Params!["ref"]!.Value<string>());
        }

        [Test]
        public void DeserializeSnapshotCommand()
        {
            var json =
                @"{""id"":""s1"",""command"":""snapshot"",""params"":{""interactive"":true,""compact"":false,""maxDepth"":5}}";
            var command = JsonConvert.DeserializeObject<AbuCommand>(json);

            Assert.IsNotNull(command);
            Assert.AreEqual("s1", command!.Id);
            Assert.AreEqual("snapshot", command.Command);
            Assert.IsNotNull(command.Params);

            var snapshotParams = command.Params!.ToObject<SnapshotParams>();
            Assert.IsNotNull(snapshotParams);
            Assert.AreEqual(true, snapshotParams!.Interactive);
            Assert.AreEqual(false, snapshotParams.Compact);
            Assert.AreEqual(5, snapshotParams.MaxDepth);
        }

        [Test]
        public void DeserializeDragCommand()
        {
            var json =
                @"{""id"":""d1"",""command"":""drag"",""params"":{""source"":""e3"",""target"":""e5""}}";
            var command = JsonConvert.DeserializeObject<AbuCommand>(json);

            Assert.IsNotNull(command);
            Assert.AreEqual("d1", command!.Id);
            Assert.AreEqual("drag", command.Command);

            var dragParams = command.Params!.ToObject<DragParams>();
            Assert.IsNotNull(dragParams);
            Assert.AreEqual("e3", dragParams!.Source);
            Assert.AreEqual("e5", dragParams.Target);
        }

        [Test]
        public void DeserializeDragCommandWithoutTarget()
        {
            var json = @"{""id"":""d2"",""command"":""drag"",""params"":{""source"":""e3""}}";
            var command = JsonConvert.DeserializeObject<AbuCommand>(json);

            Assert.IsNotNull(command);
            var dragParams = command!.Params!.ToObject<DragParams>();
            Assert.IsNotNull(dragParams);
            Assert.AreEqual("e3", dragParams!.Source);
            Assert.IsNull(dragParams.Target);
        }

        [Test]
        public void DeserializeHoverCommand()
        {
            var json = @"{""id"":""h1"",""command"":""hover"",""params"":{""ref"":""e7""}}";
            var command = JsonConvert.DeserializeObject<AbuCommand>(json);

            Assert.IsNotNull(command);
            Assert.AreEqual("h1", command!.Id);
            Assert.AreEqual("hover", command.Command);

            var refParams = command.Params!.ToObject<RefParams>();
            Assert.IsNotNull(refParams);
            Assert.AreEqual("e7", refParams!.Ref);
        }

        [Test]
        public void DeserializeScreenshotCommand()
        {
            var json = @"{""id"":""ss1"",""command"":""screenshot"",""params"":{}}";
            var command = JsonConvert.DeserializeObject<AbuCommand>(json);

            Assert.IsNotNull(command);
            Assert.AreEqual("ss1", command!.Id);
            Assert.AreEqual("screenshot", command.Command);
            Assert.IsNotNull(command.Params);
        }

        [Test]
        public void SceneNodeSerializationStructure()
        {
            var node = new AbuSceneNode
            {
                Role = "application",
                Label = "Dreamtides",
                Interactive = false,
                Children = new List<AbuSceneNode>
                {
                    new AbuSceneNode
                    {
                        Role = "button",
                        Label = "End Turn",
                        Interactive = true,
                        Children = new List<AbuSceneNode>(),
                    },
                    new AbuSceneNode
                    {
                        Role = "group",
                        Label = "Hand",
                        Interactive = false,
                        Children = new List<AbuSceneNode>
                        {
                            new AbuSceneNode
                            {
                                Role = "button",
                                Label = "Lightning Bolt",
                                Interactive = true,
                                Children = new List<AbuSceneNode>(),
                            },
                        },
                    },
                },
            };

            var json = JsonConvert.SerializeObject(node);
            var parsed = JObject.Parse(json);

            Assert.AreEqual("application", parsed["role"]!.Value<string>());
            Assert.AreEqual("Dreamtides", parsed["label"]!.Value<string>());
            Assert.AreEqual(false, parsed["interactive"]!.Value<bool>());

            var children = parsed["children"] as JArray;
            Assert.IsNotNull(children);
            Assert.AreEqual(2, children!.Count);

            Assert.AreEqual("button", children[0]["role"]!.Value<string>());
            Assert.AreEqual("End Turn", children[0]["label"]!.Value<string>());
            Assert.AreEqual(true, children[0]["interactive"]!.Value<bool>());

            var handChildren = children[1]["children"] as JArray;
            Assert.IsNotNull(handChildren);
            Assert.AreEqual(1, handChildren!.Count);
            Assert.AreEqual("Lightning Bolt", handChildren[0]["label"]!.Value<string>());
        }

        [Test]
        public void SceneNodeNullLabelOmitted()
        {
            var node = new AbuSceneNode
            {
                Role = "generic",
                Label = null,
                Interactive = false,
                Children = new List<AbuSceneNode>(),
            };

            var json = JsonConvert.SerializeObject(node);
            var parsed = JObject.Parse(json);

            Assert.AreEqual("generic", parsed["role"]!.Value<string>());
            Assert.IsFalse(parsed.ContainsKey("label"));
        }
    }
}
