import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { HttpHeaders } from '@angular/common/http';

import { Observable, throwError, of } from 'rxjs';
import { catchError, retry } from 'rxjs/operators';


import { Axis } from './axis';
import { MoveAxisRelMsg } from './move-axis-rel';
import { SurfaceGrinderCutParams } from './surface-grinder-cut-params';



@Injectable({
	providedIn: 'root'
})
export class MotorControlService {

	private moveAxisRelUrl = "api/moveAxisRel";
	private spindlePowerUrl = "api/spindlePower";
	private startSurfaceGrinderCutUrl = "api/startSurfaceGrinderCut";
	private stopUrl = "api/stop";

	httpOptions = {
		headers: new HttpHeaders({"Content-Type": "application/json"})
	};

	constructor(private http: HttpClient) { }

	moveAxisRel(axis: Axis, distance: number) : Observable<any> {
		let msg = new MoveAxisRelMsg(axis, distance);
		return this.http.post<MoveAxisRelMsg>(this.moveAxisRelUrl, msg, this.httpOptions)
			.pipe(
				catchError(this.handleError('moveAxisRel', msg))
			);
	}

	setSpindlePower(on: boolean) : Observable<any> {
		return this.http.post<boolean>(this.spindlePowerUrl, on, this.httpOptions)
			.pipe(
				catchError(this.handleError('setSpindlePower', on))
			);
	}

	startSurfaceGrinderCut(msg: SurfaceGrinderCutParams) : Observable<any> {
		return this.http.post<SurfaceGrinderCutParams>(this.startSurfaceGrinderCutUrl, msg, this.httpOptions)
			.pipe(
				catchError(this.handleError('startSurfaceGrinderCut', msg))
			);
	}

	stop() : Observable<any> {
		let msg = null;
		return this.http.post(this.stopUrl, msg, this.httpOptions)
			.pipe(
				catchError(this.handleError('stop', msg))
			);
	}

	/**
	 * Handle Http operation that failed.
	 * Let the app continue.
	 * @param operation - name of the operation that failed
	 * @param result - optional value to return as the observable result
	 */
	private handleError<T>(operation = 'operation', result?: T) {
		return (error: any): Observable<T> => {

			// TODO: send the error to remote logging infrastructure
			console.error(error); // log to console instead

			// TODO: better job of transforming error for user consumption
			console.log(`${operation} failed: ${error.message}`);

			// Let the app keep running by returning an empty result.
			return of(result as T);
		};
	}
}
