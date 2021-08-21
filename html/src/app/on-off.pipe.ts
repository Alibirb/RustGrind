import { Pipe, PipeTransform } from "@angular/core";



@Pipe({name: 'onOff'})
export class OnOffPipe implements PipeTransform {
	transform(value: boolean): string {
		return value ? "on" : "off";
	}
}
