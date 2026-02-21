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

        let timelines = model.get_all_time_lines();
        let mut prev_bpm: Option<f64> = None;
        for (i, tl) in timelines.iter().enumerate() {
            if tl.get_section_line() {
                sections.push(i);
            }
            let compare_bpm = prev_bpm.unwrap_or(model.get_bpm());
            if tl.get_bpm() != compare_bpm {
                bpms.push(i);
            }
            if tl.get_stop() != 0 {
                stops.push(i);
            }
            prev_bpm = Some(tl.get_bpm());
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

    pub fn get_sections(&self) -> &[usize] {
        &self.sections
    }

    pub fn get_bpm_changes(&self) -> &[usize] {
        &self.bpms
    }

    pub fn get_stops(&self) -> &[usize] {
        &self.stops
    }

    pub fn get_section(&mut self) -> Option<usize> {
        if self.sectionseekpos < self.sections.len() {
            let pos = self.sectionseekpos;
            self.sectionseekpos += 1;
            Some(self.sections[pos])
        } else {
            None
        }
    }

    pub fn get_bpm(&mut self) -> Option<usize> {
        if self.bpmseekpos < self.bpms.len() {
            let pos = self.bpmseekpos;
            self.bpmseekpos += 1;
            Some(self.bpms[pos])
        } else {
            None
        }
    }

    pub fn get_stop(&mut self) -> Option<usize> {
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
            && timelines[self.sections[self.sectionbasepos + 1]].get_time() > time
        {
            self.sectionbasepos += 1;
        }
        while self.sectionbasepos > 0
            && timelines[self.sections[self.sectionbasepos]].get_time() < time
        {
            self.sectionbasepos -= 1;
        }
        while self.bpmbasepos < self.bpms.len() - 1
            && timelines[self.bpms[self.bpmbasepos + 1]].get_time() > time
        {
            self.bpmbasepos += 1;
        }
        while self.bpmbasepos > 0 && timelines[self.bpms[self.bpmbasepos]].get_time() < time {
            self.bpmbasepos -= 1;
        }
        while self.stopbasepos < self.stops.len() - 1
            && timelines[self.stops[self.stopbasepos + 1]].get_time() > time
        {
            self.stopbasepos += 1;
        }
        while self.stopbasepos > 0 && timelines[self.stops[self.stopbasepos]].get_time() < time {
            self.stopbasepos -= 1;
        }
        self.sectionseekpos = self.sectionbasepos;
        self.bpmseekpos = self.bpmbasepos;
        self.stopseekpos = self.stopbasepos;
    }
}
