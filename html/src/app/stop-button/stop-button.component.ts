import { Component, OnInit } from '@angular/core';

import { MotorControlService } from '../motor-control.service';



@Component({
	selector: 'app-stop-button',
	templateUrl: './stop-button.component.html',
	styleUrls: ['./stop-button.component.scss']
})
export class StopButtonComponent implements OnInit {

	constructor(private motorControlService: MotorControlService) { }

	ngOnInit(): void {
	}

	stop(): void {
		this.motorControlService.stop().subscribe();
	}

}
