import { Component, OnInit, Input } from '@angular/core';

import { Axis } from "../axis";
import { MotorControlService } from '../motor-control.service';



@Component({
	selector: 'app-move-axis-button',
	templateUrl: './move-axis-button.component.html',
	styleUrls: ['./move-axis-button.component.scss']
})
export class MoveAxisButtonComponent implements OnInit {

	@Input() axis: Axis = Axis.X;
	@Input() distance: number = 0;

	constructor(private motorControlService: MotorControlService) { }

	ngOnInit(): void {
	}

	moveAxis(): void {
		this.motorControlService.moveAxisRel(this.axis, this.distance).subscribe();
	}

}
