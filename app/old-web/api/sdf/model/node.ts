import { IEntity, Entity } from "@/api/sdf/model/entity";
import { ISystem, System } from "@/api/sdf/model/system";
import { ISiStorable } from "@/api/sdf/model/siStorable";

import _ from "lodash";

export type INodeObject = IEntity | ISystem;
export type NodeObject = Entity | System;

export interface NodePosition {
  [key: string]: {
    x: string;
    y: string;
  };
}

export interface INode {
  id: string;
  objectType: string;
  positions: NodePosition;
  siStorable: ISiStorable;
}

export class Node implements INode {
  id: INode["id"];
  positions: NodePosition;
  objectType: INode["objectType"];
  siStorable: INode["siStorable"];

  constructor(args: INode) {
    this.id = args.id;
    this.positions = args.positions;
    this.objectType = args.objectType;
    this.siStorable = args.siStorable;
  }
}