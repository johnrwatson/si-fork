import {
  PropEnum,
  PropObject,
  PropText,
  PropLink,
  PropNumber,
  PropMap,
} from "../../components/prelude";

import { registry } from "../../registry";

/**
 * Field model for UI
 *
 * Alex's Notes...
 *
 * c.fields.addText({
 *   name: field ID
 *   label: UI display name
 *   description: short description of this field
 *   tooltip: quick info mouseover
 *   documentation: link to native docs
 * })
 *
 */

registry.base({
  typeName: "kubernetesMetadata",
  displayTypeName: "Kubernetes Meta Data",
  serviceName: "kubernetes",
  options(c) {
    c.fields.addText({
      name: "name",
      label: "Name",
      options(p: PropText) {
        p.required = true;
      },
    });
    c.fields.addText({
      name: "namespace",
      label: "Namespace",
      options(p: PropText) {
        p.required = true;
      },
    });
    c.fields.addMap({
      name: "labels",
      label: "Labels",
      // options(p: PropMap) {
      //   p.repeated = true;
      // }
    });
  },
});

registry.base({
  typeName: "kubernetesSelector",
  displayTypeName: "Kubernetes Label Selector",
  serviceName: "kubernetes",
  options(c) {
    c.fields.addMap({
      name: "matchLabels",
      label: "Match Labels",
    });
  },
});

registry.base({
  typeName: "kubernetesContainer",
  displayTypeName: "Kubernetes Container Definition",
  serviceName: "kubernetes",
  options(c) {
    c.fields.addText({
      name: "name",
      label: "Name",
    });
    c.fields.addText({
      name: "image",
      label: "Image",
    });
    c.fields.addLink({
      name: "ports",
      label: "Ports",
      options(p: PropLink) {
        p.repeated = true;
        p.lookup = {
          typeName: "kubernetesContainerPort",
        };
      },
    });
  },
});

registry.base({
  typeName: "kubernetesContainerPort",
  displayTypeName: "Kubernetes Container Port Definition",
  serviceName: "kubernetes",
  options(c) {
    c.fields.addNumber({
      name: "containerPort",
      label: "Container Port",
      options(p: PropNumber) {
        p.numberKind = "int32";
      },
    });
    c.fields.addText({
      name: "hostIp", // disabled auto/camelcase in graphql.ts for testing ...
      // name: "hostIP",
      label: "Host IP",
      options(p: PropText) {
        p.hidden = true;
      },
    });
    // temporary commenting out because it must be a number not a string.
    // need to clear empty fields before submitting..
    // c.fields.addNumber({
    //   name: "hostPort",
    //   label: "Host Port",
    //   options(p: PropNumber) {
    //     p.numberKind = "int32";
    //     p.hidden = false;
    //   },
    // });
    c.fields.addText({
      name: "name",
      label: "Name",
      options(p: PropText) {
        p.hidden = true;
      },
    });
    c.fields.addText({
      name: "protocol",
      label: "Protocol",
    });
  },
});

registry.base({
  typeName: "kubernetesPodSpec",
  displayTypeName: "Kubernetes Pod Spec",
  serviceName: "kubernetes",
  options(c) {
    c.fields.addObject({
      name: "imagePullSecrets",
      label: "Image Pull Secrets",
      options(p: PropObject) {
        p.repeated = true;
        p.properties.addText({
          name: "name",
          label: "name",
        });
      },
    });
    c.fields.addLink({
      name: "containers",
      label: "Containers",
      options(p: PropLink) {
        p.repeated = true;
        p.lookup = {
          typeName: "kubernetesContainer",
        };
      },
    });
  },
});

registry.base({
  typeName: "kubernetesPodTemplateSpec",
  displayTypeName: "Kubernetes Pod Template Spec",
  serviceName: "kubernetes",
  options(c) {
    c.fields.addLink({
      name: "metadata",
      label: "Meta Data",
      options(p: PropLink) {
        p.lookup = {
          typeName: "kubernetesMetadata",
        };
      },
    });
    c.fields.addLink({
      name: "spec",
      label: "Pod Spec",
      options(p: PropLink) {
        p.lookup = {
          typeName: "kubernetesPodSpec",
        };
      },
    });
  },
});

registry.base({
  typeName: "kubernetesLoadBalancerStatus",
  displayTypeName: "Kubernetes Load Balancer Status",
  serviceName: "kubernetes",
  options(c) {
    c.fields.addObject({
      name: "ingress",
      label: "Load Balancer Ingress",
      options(p: PropObject) {
        p.repeated = true;
        p.properties.addText({
          name: "hostname",
          label: "Hostname",
        });
        p.properties.addText({
          name: "ip",
          label: "IP",
        });
      },
    });
  },
});
