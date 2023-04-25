/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/ban-ts-comment */

import PdClient from "node-pagerduty";
import * as os from "os";

export class Pager {
  static async sendEvent(
    routingKey: string,
    severity: string,
    summary: string,
    customDetails: any = {}
  ): Promise<void> {
    const pdClient = new PdClient(routingKey);
    Object.assign(customDetails, {
      source: os.hostname(),
      group: process.env.CLUSTER ?? "",
      client: os.hostname(),
    });
    const payload = {
      payload: {
        summary: summary,
        timestamp: new Date().toISOString(),
        source: os.hostname(),
        severity: severity,
        group: process.env.CLUSTER ?? "",
        custom_details: customDetails,
      },
      routing_key: routingKey,
      event_action: "trigger",
      client: os.hostname(),
    };
    console.info("Event sending to pagerduty:", payload);
    await pdClient.events.sendEvent(payload);
  }
}
