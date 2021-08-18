import { BrowserModule } from '@angular/platform-browser';
import { FormsModule } from '@angular/forms';
import { HttpClientModule } from '@angular/common/http';
import { NgModule } from '@angular/core';

import { AppComponent } from './app.component';
import { AppRoutingModule } from './app-routing.module';
import { MoveAxisButtonComponent } from './move-axis-button/move-axis-button.component';
import { StopButtonComponent } from './stop-button/stop-button.component';
import { SurfaceGrinderCutParamsFormComponent } from './surface-grinder-cut-params-form/surface-grinder-cut-params-form.component';

@NgModule({
	declarations: [
		AppComponent,
		MoveAxisButtonComponent,
		StopButtonComponent,
		SurfaceGrinderCutParamsFormComponent,
	],
	imports: [
		AppRoutingModule,
		BrowserModule,
		FormsModule,
		HttpClientModule,
	],
	providers: [],
	bootstrap: [AppComponent]
})
export class AppModule { }
