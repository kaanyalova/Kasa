type DownloaderRowProps = {
	fileSize: number;
	downloadedSize: number;
	downloadSpeed: number;
	url: string;
	extractorName: string;
	didFail: boolean?;
	failureReason: string?;
};
