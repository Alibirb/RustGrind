import { Component, OnInit } from '@angular/core';

import { Axis } from '../axis';



@Component({
	selector: 'app-manual-control',
	templateUrl: './manual-control.component.html',
	styleUrls: ['./manual-control.component.scss']
})
export class ManualControlComponent implements OnInit {
	Axis = Axis;

	constructor() { }

	ngOnInit(): void { }
}
