import { BrowserModule } from '@angular/platform-browser';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { HttpClientModule } from '@angular/common/http';
import { NgModule } from '@angular/core';

import { AppComponent } from './app.component';
import { AppRoutingModule } from './app-routing.module';
import { ManualControlComponent } from './manual-control/manual-control.component';
import { MoveAxisButtonComponent } from './move-axis-button/move-axis-button.component';
import { MoveAxisRowComponent } from './move-axis-row/move-axis-row.component';
import { OnOffPipe } from './on-off.pipe';
import { SpindleControlButtonComponent } from './spindle-control-button/spindle-control-button.component';
import { StartHomingButtonComponent } from './start-homing-button/start-homing-button.component';
import { StopButtonComponent } from './stop-button/stop-button.component';
import { SurfaceCutControlComponent } from './surface-cut-control/surface-cut-control.component';
import { SurfaceGrinderCutParamsFormComponent } from './surface-grinder-cut-params-form/surface-grinder-cut-params-form.component';

@NgModule({
	declarations: [
		AppComponent,
		ManualControlComponent,
		MoveAxisButtonComponent,
		MoveAxisRowComponent,
		OnOffPipe,
		SpindleControlButtonComponent,
		StartHomingButtonComponent,
		StopButtonComponent,
		SurfaceCutControlComponent,
		SurfaceGrinderCutParamsFormComponent,
	],
	imports: [
		AppRoutingModule,
		BrowserModule,
		CommonModule,
		FormsModule,
		HttpClientModule,
	],
	providers: [],
	bootstrap: [AppComponent]
})
export class AppModule { }
