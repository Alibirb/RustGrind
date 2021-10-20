import { Component, OnInit } from '@angular/core';

import { MotorControlService } from '../motor-control.service';



@Component({
	selector: 'app-start-homing-button',
	templateUrl: './start-homing-button.component.html',
	styleUrls: ['./start-homing-button.component.scss']
})
export class StartHomingButtonComponent implements OnInit {

	constructor(private motorControlService: MotorControlService) { }

	ngOnInit(): void {
	}

	startHoming(): void {
		this.motorControlService.startHoming().subscribe();
	}

}
