import { Component } from '@angular/core';

import { Axis } from './axis';

@Component({
	selector: 'app-root',
	templateUrl: './app.component.html',
	styleUrls: ['./app.component.scss']
})
export class AppComponent {
	title = 'rust-grind';
	Axis = Axis;
}
