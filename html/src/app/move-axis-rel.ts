import { Axis } from './axis';



export class MoveAxisRelMsg {
	axis: Axis;
	distance: number;

	constructor(axis: Axis, distance: number) {
		this.axis = axis;
		this.distance = distance;
	}
}
