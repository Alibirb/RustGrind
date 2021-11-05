import { Component, Input, OnInit } from '@angular/core';

import { Axis } from "../axis";
import { MotorControlService } from '../motor-control.service';



@Component({
	selector: 'app-move-axis-row',
	templateUrl: './move-axis-row.component.html',
	styleUrls: ['./move-axis-row.component.scss']
})
export class MoveAxisRowComponent implements OnInit {
	@Input() axis: Axis = Axis.X;
	stepSize: number = 0.001;
	speed: number = 0.1;
	position = 0;


	constructor(private motorControlService: MotorControlService) {}

	ngOnInit(): void {}

	moveAxis(distance: number): void {
		this.motorControlService.moveAxisRel(this.axis, distance).subscribe();
	}
}
