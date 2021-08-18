import { Component, OnInit } from '@angular/core';

import { MotorControlService } from '../motor-control.service';
import { SurfaceGrinderCutParams } from '../surface-grinder-cut-params';



@Component({
	selector: 'app-surface-grinder-cut-params-form',
	templateUrl: './surface-grinder-cut-params-form.component.html',
	styleUrls: ['./surface-grinder-cut-params-form.component.scss']
})
export class SurfaceGrinderCutParamsFormComponent {

	model = new SurfaceGrinderCutParams();

	constructor(private motorControlService: MotorControlService) { }

	onSubmit() {
		this.motorControlService.startSurfaceGrinderCut(this.model).subscribe();
	}

}
