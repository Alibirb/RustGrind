import { Axis } from './axis';



export class MoveAxisRelMsg {
	axis: Axis;
	distance: number;
	speed: number;

	constructor(axis: Axis, distance: number, speed: number) {
		this.axis = axis;
		this.distance = distance;
		this.speed = speed;
	}
}
