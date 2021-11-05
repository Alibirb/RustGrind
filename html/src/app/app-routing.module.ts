import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';

import { ManualControlComponent } from './manual-control/manual-control.component';
import { SurfaceCutControlComponent } from './surface-cut-control/surface-cut-control.component';



const routes: Routes = [
	{path: 'manual-control', component: ManualControlComponent},
	{path: 'surface-cut', component: SurfaceCutControlComponent}
];

@NgModule({
	imports: [RouterModule.forRoot(routes)],
	exports: [RouterModule]
})
export class AppRoutingModule { }
