import { Component, OnInit, Input } from '@angular/core';

import { MotorControlService } from '../motor-control.service';



@Component({
	selector: 'app-spindle-control-button',
	templateUrl: './spindle-control-button.component.html',
	styleUrls: ['./spindle-control-button.component.scss']
})
export class SpindleControlButtonComponent implements OnInit {
	@Input() power: boolean = false;

	constructor(private motorControlService: MotorControlService) { }

	ngOnInit(): void {
	}

	setPower(): void {
		this.motorControlService.setSpindlePower(this.power).subscribe();
	}

}
