use crate::bms_model::BMSModel;
use crate::time_line::TimeLine;

pub struct EventLane {
    sections: Vec<usize>,
    sectionbasepos: usize,
    sectionseekpos: usize,
    bpms: Vec<usize>,
    bpmbasepos: usize,
    bpmseekpos: usize,
    stops: Vec<usize>,
    stopbasepos: usize,
    stopseekpos: usize,
}

impl EventLane {
    pub fn new(model: &BMSModel) -> Self {
        let mut sections = Vec::new();
        let mut bpms = Vec::new();
        let mut stops = Vec::new();

        let timelines = &model.timelines;
        let mut prev_bpm: Option<f64> = None;
        for (i, tl) in timelines.iter().enumerate() {
            if tl.section_line {
                sections.push(i);
            }
            let compare_bpm = prev_bpm.unwrap_or(model.bpm);
            if tl.bpm != compare_bpm {
                bpms.push(i);
            }
            if tl.stop() != 0 {
                stops.push(i);
            }
            prev_bpm = Some(tl.bpm);
        }

        EventLane {
            sections,
            sectionbasepos: 0,
            sectionseekpos: 0,
            bpms,
            bpmbasepos: 0,
            bpmseekpos: 0,
            stops,
            stopbasepos: 0,
            stopseekpos: 0,
        }
    }

    pub fn sections(&self) -> &[usize] {
        &self.sections
    }

    pub fn bpm_changes(&self) -> &[usize] {
        &self.bpms
    }

    pub fn stops(&self) -> &[usize] {
        &self.stops
    }

    pub fn section(&mut self) -> Option<usize> {
        if self.sectionseekpos < self.sections.len() {
            let pos = self.sectionseekpos;
            self.sectionseekpos += 1;
            Some(self.sections[pos])
        } else {
            None
        }
    }

    pub fn bpm(&mut self) -> Option<usize> {
        if self.bpmseekpos < self.bpms.len() {
            let pos = self.bpmseekpos;
            self.bpmseekpos += 1;
            Some(self.bpms[pos])
        } else {
            None
        }
    }

    pub fn stop(&mut self) -> Option<usize> {
        if self.stopseekpos < self.stops.len() {
            let pos = self.stopseekpos;
            self.stopseekpos += 1;
            Some(self.stops[pos])
        } else {
            None
        }
    }

    pub fn reset(&mut self) {
        self.sectionseekpos = self.sectionbasepos;
        self.bpmseekpos = self.bpmbasepos;
        self.stopseekpos = self.stopbasepos;
    }

    pub fn mark(&mut self, time: i32, timelines: &[TimeLine]) {
        while self.sectionbasepos < self.sections.len() - 1
            && timelines[self.sections[self.sectionbasepos + 1]].time() > time
        {
            self.sectionbasepos += 1;
        }
        while self.sectionbasepos > 0 && timelines[self.sections[self.sectionbasepos]].time() < time
        {
            self.sectionbasepos -= 1;
        }
        while self.bpmbasepos < self.bpms.len() - 1
            && timelines[self.bpms[self.bpmbasepos + 1]].time() > time
        {
            self.bpmbasepos += 1;
        }
        while self.bpmbasepos > 0 && timelines[self.bpms[self.bpmbasepos]].time() < time {
            self.bpmbasepos -= 1;
        }
        while self.stopbasepos < self.stops.len() - 1
            && timelines[self.stops[self.stopbasepos + 1]].time() > time
        {
            self.stopbasepos += 1;
        }
        while self.stopbasepos > 0 && timelines[self.stops[self.stopbasepos]].time() < time {
            self.stopbasepos -= 1;
        }
        self.sectionseekpos = self.sectionbasepos;
        self.bpmseekpos = self.bpmbasepos;
        self.stopseekpos = self.stopbasepos;
    }
}
