async function deleteResource(arg) {
  return {
    name: "si:awsIngressDeleteWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsIngressDeleteCommand",
        args: [arg],
      },
    ],
  };
}